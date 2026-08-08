<?php
// Spec 152E authority reconciliation: a bounded reconciliation command/job that
// compares canonical EDD truth (customers, orders, licenses, subscriptions) against
// the authority account registry, the lifecycle projections, the signed outbox event
// journal, and the verified-access lease/node registries; detects missing, duplicate,
// and stale links; repairs ONLY evidence-safe projections; and quarantines ambiguous
// or conflicting records with an exact reason.
//
//   - Missing callbacks cannot leave stale access permanently active: a canonical EDD
//     terminal transition (refunded/revoked/cancelled/failed/expired/suspended) with no
//     matching lifecycle projection is projected through the Spec 152E lifecycle
//     projector (strictly monotonic authority sequence; terminal states never
//     reactivate), and a canonical surface change with no matching signed outbox event
//     is appended as a bounded signed envelope (order, license, subscription, lease,
//     and node surfaces).
//   - Repairs are evidence-safe and preservation-only: they are derived exclusively
//     from canonical EDD rows and the authority registries, never from client input;
//     they never delete a row, never lower the authority sequence, and never flip a
//     terminal license state back to active.
//   - Ambiguous/conflicting records are quarantined with an exact bounded reason:
//     customers with no verified authority account are never promoted (raw email match
//     alone never transfers ownership), duplicate/conflicting account links require
//     operator merge review, synthetic or unverifiable records are quarantined unless
//     separately approved, and terminal-reactivation conflicts fail closed.
//   - The job is dry-run/apply, idempotent, and converges: repeated apply runs repair
//     every safe fixture exactly once and leave only the stable quarantine set; a dry
//     run applies nothing.
//   - No raw email, payment secret, license key, or unmasked real-email evidence is
//     accepted or stored; no client-controlled price/amount/grant/feature/limit/tier/
//     download field is accepted; refund/revoke/sequence truth is never rolled back.
//
// Requires docs/contracts/spec152e-authority-account.v1.php,
// docs/contracts/spec152e-edd-lifecycle-projection.v1.php, and
// docs/contracts/spec152e-authority-outbox.v1.php to be loaded first.
declare(strict_types=1);

final class FocusaSpec152eAuthorityReconciliationMigration
{
    public const SCHEMA = 'focusa.spec152e.authority_reconciliation.v1';
    public const VERSION = 1;

    public function __construct(private PDO $db, private string $prefix = 'wp_')
    {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    public function migrate(string $appliedAt, array $provenance): void
    {
        self::assertTimestamp($appliedAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encoded = self::encodeCanonical($provenance);
        $runs = $this->table('wpuiai_reconciliation_runs');
        $findings = $this->table('wpuiai_reconciliation_findings');
        $repairs = $this->table('wpuiai_reconciliation_repairs');
        $quarantine = $this->table('wpuiai_reconciliation_quarantine');
        $migrations = $this->table('wpuiai_reconciliation_schema_migrations');
        $events = $this->table('wpuiai_reconciliation_schema_events');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

        $this->db->exec("CREATE TABLE IF NOT EXISTS {$runs} (
            run_uuid {$key} NOT NULL PRIMARY KEY,
            mode VARCHAR(8) NOT NULL CHECK (mode IN ('dry_run','apply')),
            started_at VARCHAR(32) NOT NULL,
            finished_at VARCHAR(32) NOT NULL,
            findings_total BIGINT NOT NULL DEFAULT 0,
            repairs_applied BIGINT NOT NULL DEFAULT 0,
            would_repair BIGINT NOT NULL DEFAULT 0,
            quarantined_new BIGINT NOT NULL DEFAULT 0,
            stable_quarantine BIGINT NOT NULL DEFAULT 0,
            converged INTEGER NOT NULL DEFAULT 0,
            result_handle VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$findings} (
            finding_uuid {$key} NOT NULL PRIMARY KEY,
            run_uuid {$key} NOT NULL,
            category VARCHAR(40) NOT NULL,
            classification VARCHAR(40) NOT NULL,
            severity VARCHAR(8) NOT NULL,
            entity_type VARCHAR(16) NOT NULL,
            entity_ref VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NULL,
            reason VARCHAR(191) NOT NULL,
            evidence_ref VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_reconciliation_findings_run
            ON {$findings} (run_uuid)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$repairs} (
            repair_uuid {$key} NOT NULL PRIMARY KEY,
            run_uuid {$key} NOT NULL,
            finding_uuid {$key} NOT NULL,
            category VARCHAR(40) NOT NULL,
            action VARCHAR(24) NOT NULL CHECK (action IN ('project_lifecycle','append_outbox_event')),
            entity_type VARCHAR(16) NOT NULL,
            entity_ref VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NULL,
            evidence_ref VARCHAR(64) NOT NULL,
            created_at VARCHAR(32) NOT NULL
        )");
        $this->db->exec("CREATE INDEX IF NOT EXISTS {$this->prefix}wpuiai_reconciliation_repairs_run
            ON {$repairs} (run_uuid)");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$quarantine} (
            quarantine_uuid {$key} NOT NULL PRIMARY KEY,
            entity_type VARCHAR(16) NOT NULL,
            entity_ref VARCHAR(64) NOT NULL,
            account_uuid {$uuid} NULL,
            reason VARCHAR(191) NOT NULL,
            created_at VARCHAR(32) NOT NULL,
            UNIQUE (entity_type, entity_ref, reason)
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$migrations} (
            schema_version BIGINT NOT NULL PRIMARY KEY,
            schema_name VARCHAR(191) NOT NULL,
            applied_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$events} (
            event_key VARCHAR(64) NOT NULL PRIMARY KEY,
            event_type VARCHAR(32) NOT NULL,
            schema_version BIGINT NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");

        $statement = $this->db->prepare("INSERT INTO {$migrations}
            (schema_version, schema_name, applied_at, migration_provenance)
            SELECT :version, :schema, :applied_at, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing_version)");
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied_at' => $appliedAt,
            ':provenance' => $encoded,
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: runs, findings, repairs, and quarantine are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $encoded = self::encodeCanonical($provenance);
        $events = $this->table('wpuiai_reconciliation_schema_events');
        $eventKey = hash('sha256', self::SCHEMA . "\nrollback_preserved\n" . $occurredAt . "\n" . $encoded);
        $statement = $this->db->prepare("INSERT INTO {$events}
            (event_key, event_type, schema_version, occurred_at, migration_provenance)
            SELECT :event_key, 'rollback_preserved', :version, :occurred_at, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$events} WHERE event_key = :existing_key)");
        $statement->execute([
            ':event_key' => $eventKey,
            ':version' => self::VERSION,
            ':occurred_at' => $occurredAt,
            ':provenance' => $encoded,
            ':existing_key' => $eventKey,
        ]);
        return ['schema' => self::SCHEMA, 'action' => 'preserve', 'event_key' => $eventKey];
    }

    public function table(string $name): string
    {
        return $this->prefix . $name;
    }

    public static function assertTimestamp(?string $timestamp, bool $nullable = false): void
    {
        if ($nullable && ($timestamp === null || $timestamp === '')) {
            return;
        }
        if (!is_string($timestamp) || preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/', $timestamp) !== 1) {
            throw new InvalidArgumentException('canonical UTC timestamp required');
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

/**
 * Bounded discrepancy classifier. Every finding is normalized to a bounded schema:
 * category, classification, severity, entity type/reference, optional account UUID,
 * an exact bounded reason, and an opaque evidence reference. Unknown categories,
 * classifications, severities, and entity types fail closed; reasons reject raw email,
 * control characters, and unbounded length; opaque refs are validated. No secret or
 * unmasked real-email evidence can enter a finding.
 */
final class FocusaSpec152eDiscrepancyClassifier
{
    public const SCHEMA = 'focusa.spec152e.discrepancy_classifier.v1';
    public const VERSION = 1;

    /** Bounded registry: category => bounded severity and classification. */
    public const CATEGORIES = [
        'missing_account_link' => ['severity' => 'warning', 'classification' => 'quarantine_evidence_required'],
        'duplicate_account_link' => ['severity' => 'critical', 'classification' => 'quarantine_conflict'],
        'conflicting_account_link' => ['severity' => 'critical', 'classification' => 'quarantine_conflict'],
        'missing_lifecycle_projection' => ['severity' => 'info', 'classification' => 'repair_projection'],
        'stale_lifecycle_projection' => ['severity' => 'warning', 'classification' => 'repair_projection'],
        'missing_outbox_event' => ['severity' => 'warning', 'classification' => 'repair_outbox'],
        'stale_lease' => ['severity' => 'critical', 'classification' => 'repair_outbox'],
        'stale_node' => ['severity' => 'critical', 'classification' => 'repair_outbox'],
        'ambiguous_record' => ['severity' => 'warning', 'classification' => 'quarantine_ambiguous'],
        'synthetic_record' => ['severity' => 'warning', 'classification' => 'quarantine_ambiguous'],
        'terminal_reactivation_conflict' => ['severity' => 'critical', 'classification' => 'quarantine_conflict'],
    ];

    public const CLASSIFICATIONS = [
        'repair_projection', 'repair_outbox', 'quarantine_evidence_required',
        'quarantine_conflict', 'quarantine_ambiguous',
    ];

    public const SEVERITIES = ['info', 'warning', 'critical'];
    public const ENTITY_TYPES = ['customer', 'order', 'license', 'subscription', 'node', 'lease', 'account', 'refund'];

    /**
     * Normalize and validate one finding. Required: category, entity_type, entity_ref,
     * reason, evidence_ref. Optional: severity (must match the category registry),
     * account_uuid (validated when present). Fail-closed codes:
     * RECONCILIATION_CATEGORY_UNKNOWN, RECONCILIATION_ENTITY_UNKNOWN,
     * RECONCILIATION_SEVERITY_UNKNOWN, RECONCILIATION_SEVERITY_MISMATCH,
     * RECONCILIATION_RAW_EMAIL_FORBIDDEN, RECONCILIATION_EVIDENCE_REQUIRED,
     * RECONCILIATION_REASON_TOO_LONG.
     */
    public function classify(array $input): array
    {
        $category = (string) ($input['category'] ?? '');
        $spec = self::CATEGORIES[$category] ?? null;
        if ($spec === null) {
            throw new DomainException('RECONCILIATION_CATEGORY_UNKNOWN');
        }
        $entityType = (string) ($input['entity_type'] ?? '');
        if (!in_array($entityType, self::ENTITY_TYPES, true)) {
            throw new DomainException('RECONCILIATION_ENTITY_UNKNOWN');
        }
        $entityRef = (string) ($input['entity_ref'] ?? '');
        if (preg_match('/^[A-Za-z0-9._:-]{1,64}$/D', $entityRef) !== 1) {
            throw new InvalidArgumentException('opaque entity reference required');
        }
        $severity = (string) ($input['severity'] ?? $spec['severity']);
        if (!in_array($severity, self::SEVERITIES, true)) {
            throw new DomainException('RECONCILIATION_SEVERITY_UNKNOWN');
        }
        if ($severity !== $spec['severity']) {
            throw new DomainException('RECONCILIATION_SEVERITY_MISMATCH');
        }
        $reason = (string) ($input['reason'] ?? '');
        $this->assertReason($reason);
        $accountUuid = $input['account_uuid'] ?? null;
        if ($accountUuid !== null) {
            $this->assertUuid((string) $accountUuid);
        }
        $evidenceRef = (string) ($input['evidence_ref'] ?? '');
        if (preg_match('/^[A-Za-z0-9._:-]{1,64}$/D', $evidenceRef) !== 1) {
            throw new InvalidArgumentException('opaque evidence reference required');
        }
        return [
            'schema' => self::SCHEMA,
            'category' => $category,
            'classification' => (string) $spec['classification'],
            'severity' => $severity,
            'entity_type' => $entityType,
            'entity_ref' => $entityRef,
            'account_uuid' => $accountUuid,
            'reason' => $reason,
            'evidence_ref' => $evidenceRef,
        ];
    }

    public function classificationOf(string $category): string
    {
        $spec = self::CATEGORIES[$category] ?? null;
        if ($spec === null) {
            throw new DomainException('RECONCILIATION_CATEGORY_UNKNOWN');
        }
        return (string) $spec['classification'];
    }

    private function assertReason(string $reason): void
    {
        if ($reason === '') {
            throw new DomainException('RECONCILIATION_EVIDENCE_REQUIRED');
        }
        if (strlen($reason) > 191) {
            throw new DomainException('RECONCILIATION_REASON_TOO_LONG');
        }
        if (str_contains($reason, '@') || preg_match('/[\r\n]/', $reason) === 1) {
            throw new DomainException('RECONCILIATION_RAW_EMAIL_FORBIDDEN');
        }
    }

    private function assertUuid(string $uuid): void
    {
        if (preg_match('/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/D', $uuid) !== 1) {
            throw new InvalidArgumentException('bounded account UUID required');
        }
    }
}

/**
 * Bounded reconciliation command/job. One run compares every configured surface and
 * produces a typed report with bounded findings, evidence-safe repairs, quarantine
 * rows, convergence state, and an immutable result handle. Dry-run applies nothing;
 * apply mode repairs only canonical-truth-derived projections and appends only signed
 * outbox events. Re-running apply converges: safe fixtures are repaired exactly once
 * and ambiguous/conflicting records stay quarantined with the same exact reason.
 */
final class FocusaSpec152eAuthorityReconciler
{
    public const SCHEMA = 'focusa.spec152e.authority_reconciliation.v1';
    public const VERSION = 1;
    public const MODES = ['dry_run', 'apply'];
    public const REPAIR_ACTIONS = ['project_lifecycle', 'append_outbox_event'];

    /** Canonical EDD status -> lifecycle transition (reconciliation view; fail closed). */
    private const ORDER_TRANSITIONS = [
        'completed' => 'complete',
        'refunded' => 'refund',
        'partly_refunded' => 'refund',
        'revoked' => 'revoke',
        'cancelled' => 'cancel',
        'failed' => 'cancel',
    ];
    private const LICENSE_TRANSITIONS = [
        'expired' => 'expire',
        'revoked' => 'revoke',
        'disabled' => 'suspend',
        'inactive' => 'suspend',
    ];
    private const SUBSCRIPTION_TRANSITIONS = [
        'active' => 'complete',
        'cancelled' => 'cancel',
        'expired' => 'expire',
        'suspended' => 'suspend',
        'failing' => 'suspend',
    ];

    private const FORBIDDEN_COMMERCE_FIELDS = [
        'price', 'amount', 'total', 'currency', 'grants', 'features', 'limits', 'tier',
        'node_limit', 'activation_limit', 'commercial_rights', 'product_name', 'download_id',
        'product_id', 'product_code', 'license_type', 'license_type_ref',
    ];

    private PDO $db;
    private FocusaSpec152eAuthorityReconciliationMigration $schema;
    private FocusaSpec152eAuthorityAccountRepository $accounts;
    private FocusaSpec152eEddLifecycleProjector $projector;
    private FocusaSpec152eEddAuthorityHook $hook;
    private FocusaSpec152eDiscrepancyClassifier $classifier;
    private string $prefix;
    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        PDO $db,
        FocusaSpec152eAuthorityReconciliationMigration $schema,
        FocusaSpec152eAuthorityAccountRepository $accounts,
        FocusaSpec152eEddLifecycleProjector $projector,
        FocusaSpec152eEddAuthorityHook $hook,
        FocusaSpec152eDiscrepancyClassifier $classifier,
        string $prefix,
        callable $clock,
    ) {
        if (preg_match('/^[A-Za-z0-9_]*$/D', $prefix) !== 1) {
            throw new InvalidArgumentException('invalid table prefix');
        }
        $this->db = $db;
        $this->schema = $schema;
        $this->accounts = $accounts;
        $this->projector = $projector;
        $this->hook = $hook;
        $this->classifier = $classifier;
        $this->prefix = $prefix;
        $this->clock = Closure::fromCallable($clock);
    }

    /**
     * Run one bounded reconciliation pass. $mode is 'dry_run' (default; applies
     * nothing) or 'apply'. $scope optionally bounds the entity surfaces to reconcile;
     * default reconciles every surface. Returns the typed report; the run, findings,
     * repairs, and quarantine rows are durably journaled in the reconciliation schema.
     */
    public function run(string $mode = 'dry_run', array $scope = []): array
    {
        if (!in_array($mode, self::MODES, true)) {
            throw new DomainException('RECONCILIATION_MODE_UNKNOWN');
        }
        $this->assertNoClientCommerceFields($scope);
        $apply = $mode === 'apply';
        $scopeTypes = $scope === [] ? FocusaSpec152eDiscrepancyClassifier::ENTITY_TYPES : array_values($scope);
        foreach ($scopeTypes as $type) {
            if (!is_string($type) || !in_array($type, FocusaSpec152eDiscrepancyClassifier::ENTITY_TYPES, true)) {
                throw new DomainException('RECONCILIATION_SCOPE_UNKNOWN');
            }
        }

        $now = ($this->clock)();
        FocusaSpec152eAuthorityReconciliationMigration::assertTimestamp($now);
        $state = [
            'apply' => $apply,
            'findings' => [],
            'repairs' => [],
            'quarantine' => [],
            'applied' => 0,
            'would_repair' => 0,
            'repairable' => 0,
            'quarantine_new' => 0,
            'stable' => 0,
        ];

        if (in_array('customer', $scopeTypes, true)) {
            $this->reconcileCustomers($state);
        }
        if (in_array('order', $scopeTypes, true)) {
            $this->reconcileOrders($state);
        }
        if (in_array('license', $scopeTypes, true)) {
            $this->reconcileLicenses($state);
        }
        if (in_array('subscription', $scopeTypes, true)) {
            $this->reconcileSubscriptions($state);
        }
        if (in_array('lease', $scopeTypes, true)) {
            $this->reconcileLeases($state);
        }
        if (in_array('node', $scopeTypes, true)) {
            $this->reconcileNodes($state);
        }

        $repairable = $state['repairable'];
        $remaining = max(0, $repairable - ($apply ? $state['applied'] : 0));
        $converged = $remaining === 0;
        $finishedAt = ($this->clock)();
        FocusaSpec152eAuthorityReconciliationMigration::assertTimestamp($finishedAt);

        $report = [
            'schema' => self::SCHEMA,
            'mode' => $mode,
            'run_uuid' => 'run_' . bin2hex(random_bytes(16)),
            'started_at' => $now,
            'finished_at' => $finishedAt,
            'summary' => [
                'findings_total' => count($state['findings']),
                'repairable' => $repairable,
                'repairs_applied' => $state['applied'],
                'would_repair' => $state['would_repair'],
                'quarantined_new' => $state['quarantine_new'],
                'stable_quarantine' => $state['stable'],
                'converged' => $converged,
            ],
            'findings' => $state['findings'],
            'repairs' => $state['repairs'],
            'quarantine' => $state['quarantine'],
            'result_handle' => '',
        ];
        $report['result_handle'] = $this->reportHandle($mode, $state['findings'], $state['repairs'], $state['quarantine']);

        $this->persistRun($report);
        return $report;
    }

    // ── Surfaces ──────────────────────────────────────────────────────

    /** EDD customers -> authority accounts: missing links are never promoted; duplicates and conflicts quarantine. */
    private function reconcileCustomers(array &$state): void
    {
        $accountsTable = $this->prefix . 'wpuiai_authority_accounts';
        $rows = $this->db->query("SELECT account_uuid, edd_customer_id, wordpress_user_id, stripe_customer_id
            FROM {$accountsTable} ORDER BY account_uuid")->fetchAll(PDO::FETCH_ASSOC);
        $byCustomer = [];
        foreach ($rows as $row) {
            $byCustomer[(int) $row['edd_customer_id']][] = $row;
        }
        $customers = $this->db->query("SELECT id FROM {$this->prefix}edd_customers")->fetchAll(PDO::FETCH_COLUMN);
        foreach ($customers as $customerIdValue) {
            $customerId = (int) $customerIdValue;
            $accountRows = $byCustomer[$customerId] ?? [];
            if ($accountRows === []) {
                $this->quarantine($state, 'missing_account_link', 'customer', (string) $customerId, null,
                    'verified mailbox plus evidence-backed purchase linkage required; raw email match alone never transfers ownership',
                    'edd_customers');
                continue;
            }
            if (count($accountRows) > 1) {
                $extra = $accountRows[1];
                $this->quarantine($state, 'duplicate_account_link', 'account', (string) $extra['account_uuid'],
                    (string) $extra['account_uuid'],
                    'ACCOUNT_MERGE_REVIEW_REQUIRED duplicate accounts share one EDD customer',
                    'authority_accounts');
            }
        }
        // Legacy install-site accounts (Spec 22.1 inventory): a legacy record that shares
        // an EDD customer with an authority account is a duplicate link requiring merge
        // review — it is never auto-merged and never promotes an unverified email.
        $legacyTable = $this->prefix . 'install_site_accounts';
        if ($this->tableExists($legacyTable)) {
            $legacyRows = $this->db->query("SELECT id, edd_customer_id FROM {$legacyTable}")->fetchAll(PDO::FETCH_ASSOC);
            foreach ($legacyRows as $legacyRow) {
                $legacyCustomer = (int) $legacyRow['edd_customer_id'];
                if (($byCustomer[$legacyCustomer] ?? []) === []) {
                    continue; // legacy record without an authority account is not a duplicate
                }
                $this->quarantine($state, 'duplicate_account_link', 'account', 'legacy_' . (string) $legacyRow['id'], null,
                    'ACCOUNT_MERGE_REVIEW_REQUIRED legacy install-site record duplicates an authority account link',
                    'install_site_accounts');
            }
        }
        // Optional reference conflicts across accounts (Stripe / WordPress) always quarantine.
        $byStripe = [];
        $byWordpress = [];
        foreach ($rows as $row) {
            if ($row['stripe_customer_id'] !== null && $row['stripe_customer_id'] !== '') {
                $byStripe[(string) $row['stripe_customer_id']][] = $row;
            }
            if ($row['wordpress_user_id'] !== null && $row['wordpress_user_id'] !== '') {
                $byWordpress[(int) $row['wordpress_user_id']][] = $row;
            }
        }
        foreach ($byStripe as $group) {
            if (count($group) > 1) {
                foreach ($group as $row) {
                    $this->quarantine($state, 'conflicting_account_link', 'account', (string) $row['account_uuid'],
                        (string) $row['account_uuid'],
                        'ACCOUNT_MERGE_REVIEW_REQUIRED shared stripe customer link conflicts across accounts',
                        'authority_accounts');
                }
            }
        }
        foreach ($byWordpress as $group) {
            if (count($group) > 1) {
                foreach ($group as $row) {
                    $this->quarantine($state, 'conflicting_account_link', 'account', (string) $row['account_uuid'],
                        (string) $row['account_uuid'],
                        'ACCOUNT_MERGE_REVIEW_REQUIRED shared wordpress user link conflicts across accounts',
                        'authority_accounts');
                }
            }
        }
    }

    /** EDD orders -> lifecycle projection + signed outbox events. */
    private function reconcileOrders(array &$state): void
    {
        $eventMap = FocusaSpec152eEddAuthorityHook::EDD_EVENT_MAP['order'];
        $orders = $this->db->query("SELECT id, status, customer_id FROM {$this->prefix}edd_orders")->fetchAll(PDO::FETCH_ASSOC);
        foreach ($orders as $order) {
            $orderId = (int) $order['id'];
            $status = strtolower((string) $order['status']);
            if (!isset(self::ORDER_TRANSITIONS[$status]) || !isset($eventMap[$status])) {
                continue;
            }
            $account = $this->accounts->findByCustomerId((int) $order['customer_id']);
            if ($account === null) {
                continue; // covered by the customer surface; unverified records are never promoted
            }
            $transition = self::ORDER_TRANSITIONS[$status];
            $conflicted = $this->reconcileLifecycleSurface(
                $state, $account, $transition, $status, 'order', 'order_id', $orderId,
                'edd_orders',
            );
            if (!$conflicted) {
                $this->reconcileMissingOutboxEvent($state, $account, $eventMap[$status], 'order_id', $orderId,
                    'edd_orders', $this->licenseIdForOrder($orderId));
            }
        }
    }

    /** EDD licenses -> lifecycle projection + signed outbox events; synthetic records quarantine. */
    private function reconcileLicenses(array &$state): void
    {
        $eventMap = FocusaSpec152eEddAuthorityHook::EDD_EVENT_MAP['license'];
        $licenses = $this->db->query("SELECT id, status, customer_id, order_id FROM {$this->prefix}edd_licenses")->fetchAll(PDO::FETCH_ASSOC);
        foreach ($licenses as $license) {
            $licenseId = (int) $license['id'];
            $status = strtolower((string) $license['status']);
            // A license without a canonical order is unverifiable (synthetic) and
            // quarantines unless separately approved, regardless of its stored status.
            if ($license['order_id'] === null || !$this->orderExists((int) $license['order_id'])) {
                $this->quarantine($state, 'synthetic_record', 'license', (string) $licenseId, null,
                    'synthetic record quarantined unless separately approved; verified identity plus matching order item required',
                    'edd_licenses');
                continue;
            }
            if (!isset(self::LICENSE_TRANSITIONS[$status])) {
                continue; // active licenses are governed by their order/subscription surface
            }
            $account = $this->accounts->findByCustomerId((int) $license['customer_id']);
            if ($account === null) {
                continue;
            }
            $transition = self::LICENSE_TRANSITIONS[$status];
            $conflicted = $this->reconcileLifecycleSurface(
                $state, $account, $transition, $status, 'license', 'license_id', $licenseId,
                'edd_licenses',
            );
            if (!$conflicted && isset($eventMap[$status])) {
                $this->reconcileMissingOutboxEvent($state, $account, $eventMap[$status], 'license_id', $licenseId,
                    'edd_licenses');
            }
        }
    }

    /** EDD subscriptions -> lifecycle projection + signed outbox events. */
    private function reconcileSubscriptions(array &$state): void
    {
        $eventMap = FocusaSpec152eEddAuthorityHook::EDD_EVENT_MAP['subscription'];
        $subscriptions = $this->db->query("SELECT id, status, customer_id FROM {$this->prefix}edd_subscriptions")->fetchAll(PDO::FETCH_ASSOC);
        foreach ($subscriptions as $subscription) {
            $subscriptionId = (int) $subscription['id'];
            $status = strtolower((string) $subscription['status']);
            if (!isset(self::SUBSCRIPTION_TRANSITIONS[$status]) || !isset($eventMap[$status])) {
                continue;
            }
            $account = $this->accounts->findByCustomerId((int) $subscription['customer_id']);
            if ($account === null) {
                continue;
            }
            $transition = self::SUBSCRIPTION_TRANSITIONS[$status];
            $conflicted = $this->reconcileLifecycleSurface(
                $state, $account, $transition, $status, 'subscription', 'subscription_id', $subscriptionId,
                'edd_subscriptions',
            );
            if (!$conflicted) {
                $this->reconcileMissingOutboxEvent($state, $account, $eventMap[$status], 'subscription_id', $subscriptionId,
                    'edd_subscriptions');
            }
        }
    }

    /** Verified-access postures (leases) -> stale leases are superseded/revoked; unverifiable leases quarantine. */
    private function reconcileLeases(array &$state): void
    {
        $postures = $this->db->query("SELECT posture_uuid, account_uuid, node_uuid, status
            FROM {$this->prefix}wpuiai_verified_access_postures")->fetchAll(PDO::FETCH_ASSOC);
        foreach ($postures as $posture) {
            if (!in_array((string) $posture['status'], ['issued', 'refreshed'], true)) {
                continue;
            }
            $leaseUuid = (string) $posture['posture_uuid'];
            $accountUuid = (string) $posture['account_uuid'];
            $boundLicense = $this->boundLicenseForLease($leaseUuid);
            if ($boundLicense === null) {
                $this->quarantine($state, 'ambiguous_record', 'lease', $leaseUuid, $accountUuid,
                    'lease has no signed issuance event; operator review required',
                    'verified_access_postures');
                continue;
            }
            if ($this->licenseIsTerminal($boundLicense)) {
                $this->reconcileStaleLease($state, $leaseUuid, $accountUuid, $boundLicense, $posture['node_uuid']);
            }
        }
    }

    /** Verified-access nodes -> stale nodes deactivated; unverifiable nodes quarantine. */
    private function reconcileNodes(array &$state): void
    {
        $nodes = $this->db->query("SELECT node_uuid, account_uuid FROM {$this->prefix}wpuiai_verified_access_nodes")->fetchAll(PDO::FETCH_ASSOC);
        foreach ($nodes as $node) {
            $nodeUuid = (string) $node['node_uuid'];
            $latest = $this->latestNodePosture($nodeUuid);
            if ($latest === null) {
                $this->quarantine($state, 'ambiguous_record', 'node', $nodeUuid, (string) $node['account_uuid'],
                    'unverified node without posture evidence requires review',
                    'verified_access_nodes');
                continue;
            }
            if (in_array($latest, ['revoked', 'superseded'], true) && !$this->outboxHasNodeEvent($nodeUuid)) {
                $account = $this->findAccountByUuidOrNull((string) $node['account_uuid']);
                if ($account === null) {
                    continue;
                }
                $finding = $this->classify('stale_node', 'node', $nodeUuid, (string) $node['account_uuid'],
                    'stale node with terminal posture must be deactivated; access must not stay active',
                    'verified_access_nodes');
                $this->scheduleOutboxRepair($state, $finding, [
                    'event_type' => 'node_deactivated',
                    'account_uuid' => (string) $account['account_uuid'],
                    'edd_customer_id' => (int) $account['edd_customer_id'],
                    'node_uuid' => $nodeUuid,
                ], 'node', $nodeUuid, 'node_deactivated', (string) $node['account_uuid']);
            }
        }
    }

    // ── Shared reconciliation steps ────────────────────────────────────

    /**
     * Compare one canonical surface row against its lifecycle projection and schedule
     * the evidence-safe projection repair. Returns true when the row conflicts with
     * existing terminal truth (quarantined; never repaired).
     */
    private function reconcileLifecycleSurface(
        array &$state,
        array $account,
        string $transition,
        string $status,
        string $surface,
        string $refColumn,
        int $refValue,
        string $evidenceRef,
    ): bool {
        $accountUuid = (string) $account['account_uuid'];
        $customerId = (int) $account['edd_customer_id'];
        $current = $this->lifecycleState($accountUuid, $refColumn, $refValue);
        $canonicalTarget = FocusaSpec152eEddLifecycleProjector::TRANSITIONS[$transition]['license_state'];

        if ($current['license_state'] === 'none') {
            if ($transition === 'complete') {
                $finding = $this->classify('missing_lifecycle_projection', $surface, (string) $refValue, $accountUuid,
                    "missing lifecycle projection for canonical {$surface} '{$status}'",
                    $evidenceRef);
                $this->scheduleProjectionRepair($state, $finding, $surface, $transition, $accountUuid, $customerId, $refColumn, $refValue);
                return false;
            }
            // Terminal state with no prior entitlement truth: never invent truth.
            $this->quarantine($state, 'ambiguous_record', $surface, (string) $refValue, $accountUuid,
                'terminal canonical state without prior entitlement truth requires review; no truth is invented',
                $evidenceRef);
            return true;
        }

        if ($current['license_state'] === $canonicalTarget) {
            return false; // projection already reflects canonical truth
        }

        if ($canonicalTarget === 'active' && in_array($current['license_state'], FocusaSpec152eEddLifecycleProjector::TERMINAL_STATES, true)) {
            $this->quarantine($state, 'terminal_reactivation_conflict', $surface, (string) $refValue, $accountUuid,
                'LICENSE_TERMINAL_REACTIVATION_DENIED canonical row would reactivate a terminal projection; operator review required',
                $evidenceRef);
            return true;
        }

        $finding = $this->classify('stale_lifecycle_projection', $surface, (string) $refValue, $accountUuid,
            "stale lifecycle projection: canonical {$surface} '{$status}' requires '{$transition}'",
            $evidenceRef);
        $this->scheduleProjectionRepair($state, $finding, $surface, $transition, $accountUuid, $customerId, $refColumn, $refValue);
        return false;
    }

    /** Repair a missing or stale lifecycle projection through the canonical projector. */
    private function scheduleProjectionRepair(
        array &$state,
        array $finding,
        string $surface,
        string $transition,
        string $accountUuid,
        int $customerId,
        string $refColumn,
        int $refValue,
    ): void {
        $state['repairable']++;
        $state['findings'][] = $finding;
        if (!$state['apply']) {
            $state['would_repair']++;
            return;
        }
        $input = [
            'surface' => $surface,
            'transition' => $transition,
            'account_uuid' => $accountUuid,
            'edd_customer_id' => $customerId,
            $refColumn => $refValue,
            'state_reason' => 'reconciled from canonical ' . $surface . ' truth',
        ];
        $idempotencyKey = 'reconcile:' . $surface . ':' . $refValue . ':' . $transition;
        $result = $this->projector->project($input + ['request_id' => 'reconcile', 'idempotency_key' => $idempotencyKey]);
        if (($result['decision'] ?? '') === 'denied') {
            // The canonical projector refused the repair (never rolls back sequence or
            // reactivates terminal truth): drop the repairable finding and quarantine.
            array_pop($state['findings']);
            $state['repairable']--;
            $this->quarantine($state, 'terminal_reactivation_conflict', $finding['entity_type'], $finding['entity_ref'], $accountUuid,
                (string) ($result['error_code'] ?? 'RECONCILIATION_REPAIR_DENIED') . ' projection repair denied by canonical projector',
                $finding['evidence_ref']);
            return;
        }
        $state['applied']++;
        $state['repairs'][] = [
            'action' => 'project_lifecycle',
            'category' => $finding['category'],
            'entity_type' => $finding['entity_type'],
            'entity_ref' => $finding['entity_ref'],
            'account_uuid' => $accountUuid,
            'evidence_ref' => $finding['evidence_ref'],
            'surface' => $surface,
            'transition' => $transition,
        ];
    }

    /** Detect a missing signed outbox event for a canonical surface row and append it. */
    private function reconcileMissingOutboxEvent(
        array &$state,
        array $account,
        string $eventType,
        string $refColumn,
        int $refValue,
        string $evidenceRef,
        ?int $boundLicenseId = null,
    ): void {
        if ($this->outboxHasEvent($eventType, $refColumn, $refValue)) {
            return;
        }
        $entityType = match ($refColumn) {
            'order_id' => 'order',
            'license_id' => 'license',
            'subscription_id' => 'subscription',
            default => 'order',
        };
        $finding = $this->classify('missing_outbox_event', $entityType, (string) $refValue, (string) $account['account_uuid'],
            "missing signed outbox event '{$eventType}' for canonical {$refColumn}",
            $evidenceRef);
        $eventInput = [
            'event_type' => $eventType,
            'account_uuid' => (string) $account['account_uuid'],
            'edd_customer_id' => (int) $account['edd_customer_id'],
            $refColumn => $refValue,
        ];
        if ($boundLicenseId !== null) {
            $eventInput['license_id'] = $boundLicenseId;
        }
        $this->scheduleOutboxRepair($state, $finding, $eventInput, $refColumn, $refValue, $eventType, (string) $account['account_uuid']);
    }

    /** Repair a stale lease: append the signed lease_superseded / lease_revoked outbox event. */
    private function reconcileStaleLease(array &$state, string $leaseUuid, string $accountUuid, int $licenseId, mixed $nodeUuid): void
    {
        if ($this->outboxHasLeaseTerminalEvent($leaseUuid)) {
            return;
        }
        $account = $this->findAccountByUuidOrNull($accountUuid);
        if ($account === null) {
            return;
        }
        $licenseStatus = $this->licenseStatus($licenseId);
        $orderStatus = $this->licenseOrderStatus($licenseId);
        $eventType = ($licenseStatus === 'revoked' || $orderStatus === 'revoked') ? 'lease_revoked' : 'lease_superseded';
        $finding = $this->classify('stale_lease', 'lease', $leaseUuid, $accountUuid,
            "stale lease bound to terminal license {$licenseId} must be " . ($eventType === 'lease_revoked' ? 'revoked' : 'superseded'),
            'verified_access_postures');
        $this->scheduleOutboxRepair($state, $finding, [
            'event_type' => $eventType,
            'account_uuid' => $accountUuid,
            'edd_customer_id' => (int) $account['edd_customer_id'],
            'lease_uuid' => $leaseUuid,
            'license_id' => $licenseId,
            'node_uuid' => $nodeUuid !== null ? (string) $nodeUuid : null,
        ], 'lease', $leaseUuid, $eventType, $accountUuid);
    }

    /** Append one signed outbox event in a caller-owned transaction (dry-run applies nothing). */
    private function scheduleOutboxRepair(array &$state, array $finding, array $eventInput, string $refKey, int|string $refValue, string $eventType, string $accountUuid): void
    {
        $state['repairable']++;
        $state['findings'][] = $finding;
        if (!$state['apply']) {
            $state['would_repair']++;
            return;
        }
        $idempotencyKey = 'reconcile:' . $refKey . ':' . $refValue . ':' . $eventType;
        try {
            $this->db->beginTransaction();
            $this->hook->append($eventInput + ['request_id' => 'reconcile', 'idempotency_key' => $idempotencyKey]);
            $this->db->commit();
        } catch (Throwable $error) {
            if ($this->db->inTransaction()) {
                $this->db->rollBack();
            }
            // The signed append failed closed: drop the repairable finding and quarantine.
            array_pop($state['findings']);
            $state['repairable']--;
            $this->quarantine($state, 'ambiguous_record', $finding['entity_type'], $finding['entity_ref'], $accountUuid,
                (string) $error->getMessage() . ' outbox append denied; repair quarantined for operator review',
                $finding['evidence_ref']);
            return;
        }
        $state['applied']++;
        $state['repairs'][] = [
            'action' => 'append_outbox_event',
            'category' => $finding['category'],
            'entity_type' => $finding['entity_type'],
            'entity_ref' => $finding['entity_ref'],
            'account_uuid' => $accountUuid,
            'evidence_ref' => $finding['evidence_ref'],
            'event_type' => $eventType,
        ];
    }

    // ── Quarantine ─────────────────────────────────────────────────────

    /** Quarantine a record with an exact reason; stable across runs. */
    private function quarantine(array &$state, string $category, string $entityType, string $entityRef, ?string $accountUuid, string $reason, string $evidenceRef): void
    {
        $finding = $this->classify($category, $entityType, $entityRef, $accountUuid, $reason, $evidenceRef);
        $state['findings'][] = $finding;
        if ($this->quarantineExists($entityType, $entityRef, $reason)) {
            $state['stable']++;
            $state['quarantine'][] = [
                'entity_type' => $entityType,
                'entity_ref' => $entityRef,
                'account_uuid' => $accountUuid,
                'reason' => $reason,
            ];
            return;
        }
        if ($state['apply']) {
            $now = ($this->clock)();
            FocusaSpec152eAuthorityReconciliationMigration::assertTimestamp($now);
            $table = $this->schema->table('wpuiai_reconciliation_quarantine');
            $statement = $this->db->prepare("INSERT INTO {$table}
                (quarantine_uuid, entity_type, entity_ref, account_uuid, reason, created_at)
                VALUES (:uuid, :type, :ref, :account, :reason, :created)");
            $statement->execute([
                ':uuid' => 'q_' . bin2hex(random_bytes(16)),
                ':type' => $entityType,
                ':ref' => $entityRef,
                ':account' => $accountUuid,
                ':reason' => $reason,
                ':created' => $now,
            ]);
        }
        $state['quarantine_new']++;
        $state['quarantine'][] = [
            'entity_type' => $entityType,
            'entity_ref' => $entityRef,
            'account_uuid' => $accountUuid,
            'reason' => $reason,
        ];
    }

    private function quarantineExists(string $entityType, string $entityRef, string $reason): bool
    {
        $table = $this->schema->table('wpuiai_reconciliation_quarantine');
        $statement = $this->db->prepare("SELECT 1 FROM {$table} WHERE entity_type = :type AND entity_ref = :ref AND reason = :reason LIMIT 1");
        $statement->execute([':type' => $entityType, ':ref' => $entityRef, ':reason' => $reason]);
        return $statement->fetchColumn() !== false;
    }

    // ── Canonical truth reads ──────────────────────────────────────────

    private function lifecycleState(string $accountUuid, string $refColumn, int $refValue): array
    {
        $table = $this->prefix . 'wpuiai_edd_lifecycle_events';
        $statement = $this->db->prepare("SELECT license_state, refresh_posture, result_sequence
            FROM {$table}
            WHERE account_uuid = :uuid AND {$refColumn} = :ref AND decision IN ('applied','replayed')
            ORDER BY result_sequence DESC, created_at DESC LIMIT 1");
        $statement->execute([':uuid' => $accountUuid, ':ref' => $refValue]);
        $row = $statement->fetch(PDO::FETCH_ASSOC);
        return $row === false
            ? ['license_state' => 'none', 'refresh_posture' => 'allowed', 'result_sequence' => 0]
            : ['license_state' => (string) $row['license_state'], 'refresh_posture' => (string) $row['refresh_posture'], 'result_sequence' => (int) $row['result_sequence']];
    }

    private function outboxHasEvent(string $eventType, string $refColumn, int $refValue): bool
    {
        $table = $this->prefix . 'wpuiai_authority_outbox';
        $statement = $this->db->prepare("SELECT 1 FROM {$table} WHERE event_type = :type AND {$refColumn} = :ref LIMIT 1");
        $statement->execute([':type' => $eventType, ':ref' => $refValue]);
        return $statement->fetchColumn() !== false;
    }

    private function outboxHasLeaseTerminalEvent(string $leaseUuid): bool
    {
        $table = $this->prefix . 'wpuiai_authority_outbox';
        $statement = $this->db->prepare("SELECT 1 FROM {$table}
            WHERE event_type IN ('lease_superseded','lease_revoked') AND lease_uuid = :lease LIMIT 1");
        $statement->execute([':lease' => $leaseUuid]);
        return $statement->fetchColumn() !== false;
    }

    private function outboxHasNodeEvent(string $nodeUuid): bool
    {
        $table = $this->prefix . 'wpuiai_authority_outbox';
        $statement = $this->db->prepare("SELECT 1 FROM {$table} WHERE event_type = 'node_deactivated' AND node_uuid = :node LIMIT 1");
        $statement->execute([':node' => $nodeUuid]);
        return $statement->fetchColumn() !== false;
    }

    private function boundLicenseForLease(string $leaseUuid): ?int
    {
        $table = $this->prefix . 'wpuiai_authority_outbox';
        $statement = $this->db->prepare("SELECT license_id FROM {$table}
            WHERE event_type IN ('lease_issued','lease_superseded','lease_revoked') AND lease_uuid = :lease AND license_id IS NOT NULL
            ORDER BY created_at DESC LIMIT 1");
        $statement->execute([':lease' => $leaseUuid]);
        $value = $statement->fetchColumn();
        return $value === false ? null : (int) $value;
    }

    private function licenseIsTerminal(int $licenseId): bool
    {
        $status = $this->licenseStatus($licenseId);
        if (in_array($status, ['expired', 'revoked', 'disabled', 'inactive'], true)) {
            return true;
        }
        return in_array($this->licenseOrderStatus($licenseId), ['refunded', 'partly_refunded', 'revoked', 'cancelled', 'failed'], true);
    }

    private function licenseStatus(int $licenseId): string
    {
        $statement = $this->db->prepare("SELECT status FROM {$this->prefix}edd_licenses WHERE id = :id");
        $statement->execute([':id' => $licenseId]);
        $value = $statement->fetchColumn();
        return $value === false ? '' : strtolower((string) $value);
    }

    private function licenseOrderStatus(int $licenseId): string
    {
        $statement = $this->db->prepare("SELECT o.status FROM {$this->prefix}edd_licenses l
            JOIN {$this->prefix}edd_orders o ON o.id = l.order_id WHERE l.id = :id");
        $statement->execute([':id' => $licenseId]);
        $value = $statement->fetchColumn();
        return $value === false ? '' : strtolower((string) $value);
    }

    private function latestNodePosture(string $nodeUuid): ?string
    {
        $table = $this->prefix . 'wpuiai_verified_access_postures';
        $statement = $this->db->prepare("SELECT status FROM {$table} WHERE node_uuid = :node ORDER BY created_at DESC LIMIT 1");
        $statement->execute([':node' => $nodeUuid]);
        $value = $statement->fetchColumn();
        return $value === false ? null : (string) $value;
    }

    private function orderExists(int $orderId): bool
    {
        $statement = $this->db->prepare("SELECT 1 FROM {$this->prefix}edd_orders WHERE id = :id LIMIT 1");
        $statement->execute([':id' => $orderId]);
        return $statement->fetchColumn() !== false;
    }

    private function licenseIdForOrder(int $orderId): ?int
    {
        $statement = $this->db->prepare("SELECT id FROM {$this->prefix}edd_licenses WHERE order_id = :order LIMIT 1");
        $statement->execute([':order' => $orderId]);
        $value = $statement->fetchColumn();
        return $value === false ? null : (int) $value;
    }

    private function tableExists(string $table): bool
    {
        $driver = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME);
        if ($driver === 'mysql') {
            $statement = $this->db->prepare('SELECT 1 FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = :name');
            $statement->execute([':name' => $table]);
            return $statement->fetchColumn() !== false;
        }
        $statement = $this->db->prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = :name");
        $statement->execute([':name' => $table]);
        return $statement->fetchColumn() !== false;
    }

    private function findAccountByUuidOrNull(string $accountUuid): ?array
    {
        try {
            return $this->accounts->findByUuid($accountUuid);
        } catch (OutOfBoundsException $error) {
            return null;
        }
    }

    // ── Report and persistence ─────────────────────────────────────────

    private function classify(string $category, string $entityType, string $entityRef, ?string $accountUuid, string $reason, string $evidenceRef): array
    {
        return $this->classifier->classify([
            'category' => $category,
            'entity_type' => $entityType,
            'entity_ref' => $entityRef,
            'account_uuid' => $accountUuid,
            'reason' => $reason,
            'evidence_ref' => $evidenceRef,
        ]);
    }

    /** Deterministic immutable handle over the semantic report (run_uuid/timestamps excluded). */
    private function reportHandle(string $mode, array $findings, array $repairs, array $quarantine): string
    {
        $semantic = static function (array $row): array {
            $keys = ['category', 'classification', 'severity', 'entity_type', 'entity_ref', 'account_uuid', 'reason', 'action', 'event_type', 'surface', 'transition'];
            $out = [];
            foreach ($keys as $key) {
                if (array_key_exists($key, $row)) {
                    $out[$key] = $row[$key];
                }
            }
            return $out;
        };
        $payload = [
            'mode' => $mode,
            'findings' => array_map($semantic, $findings),
            'repairs' => array_map($semantic, $repairs),
            'quarantine' => array_map($semantic, $quarantine),
        ];
        return hash('sha256', FocusaSpec152eAuthorityReconciliationMigration::encodeCanonical($payload));
    }

    private function persistRun(array $report): void
    {
        $table = $this->schema->table('wpuiai_reconciliation_runs');
        $now = ($this->clock)();
        FocusaSpec152eAuthorityReconciliationMigration::assertTimestamp($now);
        $statement = $this->db->prepare("INSERT INTO {$table}
            (run_uuid, mode, started_at, finished_at, findings_total, repairs_applied, would_repair,
             quarantined_new, stable_quarantine, converged, result_handle, migration_provenance)
            VALUES (:run, :mode, :started, :finished, :findings, :applied, :would_repair,
                    :quarantined_new, :stable, :converged, :handle, :provenance)");
        $statement->execute([
            ':run' => $report['run_uuid'],
            ':mode' => $report['mode'],
            ':started' => $report['started_at'],
            ':finished' => $report['finished_at'],
            ':findings' => $report['summary']['findings_total'],
            ':applied' => $report['summary']['repairs_applied'],
            ':would_repair' => $report['summary']['would_repair'],
            ':quarantined_new' => $report['summary']['quarantined_new'],
            ':stable' => $report['summary']['stable_quarantine'],
            ':converged' => $report['summary']['converged'] ? 1 : 0,
            ':handle' => $report['result_handle'],
            ':provenance' => FocusaSpec152eAuthorityReconciliationMigration::encodeCanonical(['source' => 'authority_reconciliation']),
        ]);
        $this->persistFindings($report['run_uuid'], $report['findings']);
        $this->persistRepairs($report['run_uuid'], $report['repairs']);
    }

    private function persistFindings(string $runUuid, array $findings): void
    {
        $table = $this->schema->table('wpuiai_reconciliation_findings');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (finding_uuid, run_uuid, category, classification, severity, entity_type, entity_ref,
             account_uuid, reason, evidence_ref, created_at)
            VALUES (:uuid, :run, :category, :classification, :severity, :type, :ref, :account, :reason, :evidence, :created)");
        foreach ($findings as $finding) {
            $statement->execute([
                ':uuid' => 'f_' . bin2hex(random_bytes(16)),
                ':run' => $runUuid,
                ':category' => $finding['category'],
                ':classification' => $finding['classification'],
                ':severity' => $finding['severity'],
                ':type' => $finding['entity_type'],
                ':ref' => $finding['entity_ref'],
                ':account' => $finding['account_uuid'],
                ':reason' => $finding['reason'],
                ':evidence' => $finding['evidence_ref'],
                ':created' => ($this->clock)(),
            ]);
        }
    }

    private function persistRepairs(string $runUuid, array $repairs): void
    {
        $table = $this->schema->table('wpuiai_reconciliation_repairs');
        $statement = $this->db->prepare("INSERT INTO {$table}
            (repair_uuid, run_uuid, finding_uuid, category, action, entity_type, entity_ref,
             account_uuid, evidence_ref, created_at)
            VALUES (:uuid, :run, :finding, :category, :action, :type, :ref, :account, :evidence, :created)");
        foreach ($repairs as $index => $repair) {
            $statement->execute([
                ':uuid' => 'r_' . bin2hex(random_bytes(16)),
                ':run' => $runUuid,
                ':finding' => 'f_' . bin2hex(random_bytes(16)),
                ':category' => $repair['category'],
                ':action' => $repair['action'],
                ':type' => $repair['entity_type'],
                ':ref' => $repair['entity_ref'],
                ':account' => $repair['account_uuid'],
                ':evidence' => $repair['evidence_ref'],
                ':created' => ($this->clock)(),
            ]);
        }
    }

    private function assertNoClientCommerceFields(array $input): void
    {
        foreach (self::FORBIDDEN_COMMERCE_FIELDS as $field) {
            if (array_key_exists($field, $input)) {
                throw new DomainException('CLIENT_COMMERCIAL_FIELDS_FORBIDDEN');
            }
        }
    }
}
