<?php
// 152E.06.05 Cut over facades and disable direct install-site issuance
// (Spec 152E §2 mandatory decisions 3, 5, 11, 13; §9 facade registry; §10
// public activation API; §11 EDD checkout; §12 Evaluation forbidden; §21
// surface consolidation; §22.3 cutover steps 7-10; §22.4 rollback; §23
// acceptance "Website paid Focusa", "Evaluation", "Facade spoof", and
// "Legacy install-site record" rows; Specs 152, 150A, 152A-D; Spec 158
// implementation excluded).
//
// The cutover gate is the single feature-gated switch that:
//   - publishes the exact cutover state: after publish, every new customer /
//     evaluator issuance route resolves ONLY to the WPUIAI.com EDD authority
//     kernel; no presenter, installer, facade, Stripe metadata, or local
//     runtime may create entitlement truth;
//   - denies the direct issuance classes that previously created truth:
//     install-site create / payment-intent / Stripe webhook issuance routes
//     and the caller-parameter WPUIAI.com custom issue route (each denied
//     with INSTALL_SITE_ISSUANCE_DISABLED), direct Stripe product flow
//     (STRIPE_DIRECT_FLOW_DENIED), and local self-Evaluation --eval
//     (LOCAL_EVALUATION_DENIED);
//   - makes the legacy install-site tables read-only (wpuiai_licenses,
//     wpuiai_license_audit): SELECT for bounded validation/recovery is the
//     only permitted operation; every mutation fails closed with
//     LEGACY_TABLE_READ_ONLY before any statement executes;
//   - retains bounded legacy validation/recovery surfaces (validate,
//     keys/validate, status, recovery status/export/diagnostics/repair/
//     update/uninstall) that read legacy state as migration input and never
//     grant entitlement;
//   - resolves every install-site facade surface to its EDD authority proxy
//     route (facade proxy only, no local issuance route); and
//   - journal-append audits every publication and denial in a replay-safe
//     digest-chained journal and exposes the published state for exact
//     cutover verification.
//
// Before the cutover is published, every route, table, and surface fails
// closed with CUTOVER_STATE_REQUIRED — nothing is issuable, mutable, or
// readable as authority. Publication is idempotent and immutable: a replay
// returns the stored state, a different payload fails closed with
// CUTOVER_STATE_ALREADY_PUBLISHED, and rollback is preservation-only.
//
// No unverified-email promotion, no local/self-issued entitlement, no
// independent facade authority, no client-controlled EDD price/grants (the
// decision signature has no price, grant, tier, limit, or feature inputs),
// and no raw email, raw key, payment id, or secret ever leaves this contract
// — only 64-hex digests and masked values.
declare(strict_types=1);

final class FocusaSpec152eAuthorityCutoverSchema
{
    public const SCHEMA = 'focusa.spec152e.authority_cutover.v1';
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
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $migrations = $this->table('wpuiai_cutover_schema_migrations');
        $events = $this->table('wpuiai_cutover_schema_events');
        $state = $this->table('wpuiai_cutover_state');
        $journal = $this->table('wpuiai_cutover_state_journal');
        $denials = $this->table('wpuiai_cutover_denials');
        $legacy = $this->table('wpuiai_cutover_legacy_tables');
        $recovery = $this->table('wpuiai_cutover_recovery_surfaces');
        $uuid = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(36)' : 'TEXT';
        $key = $this->db->getAttribute(PDO::ATTR_DRIVER_NAME) === 'mysql' ? 'VARCHAR(191)' : 'TEXT';

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
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$state} (
            state_key VARCHAR(64) NOT NULL PRIMARY KEY,
            cutover_version VARCHAR(64) NOT NULL,
            effective_at VARCHAR(32) NOT NULL,
            state_payload TEXT NOT NULL,
            state_digest VARCHAR(64) NOT NULL,
            published_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$journal} (
            journal_seq BIGINT NOT NULL PRIMARY KEY,
            journal_key VARCHAR(64) NOT NULL UNIQUE,
            event_type VARCHAR(32) NOT NULL,
            state_key VARCHAR(64) NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            detail TEXT NOT NULL,
            previous_digest VARCHAR(64) NOT NULL,
            entry_digest VARCHAR(64) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$denials} (
            denial_uuid {$uuid} NOT NULL PRIMARY KEY,
            denial_key VARCHAR(64) NOT NULL UNIQUE,
            surface VARCHAR(64) NOT NULL,
            route {$key} NOT NULL,
            denial_code VARCHAR(64) NOT NULL,
            next_action {$key} NOT NULL,
            request_id {$key} NOT NULL,
            idempotency_key {$key} NOT NULL,
            occurred_at VARCHAR(32) NOT NULL,
            migration_provenance TEXT NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$legacy} (
            table_name {$key} NOT NULL PRIMARY KEY,
            read_only VARCHAR(16) NOT NULL,
            allow_operations TEXT NOT NULL,
            classification VARCHAR(191) NOT NULL,
            migration VARCHAR(191) NOT NULL
        )");
        $this->db->exec("CREATE TABLE IF NOT EXISTS {$recovery} (
            surface VARCHAR(64) NOT NULL PRIMARY KEY,
            route {$key} NOT NULL,
            retained_for VARCHAR(32) NOT NULL,
            grants_entitlement VARCHAR(16) NOT NULL
        )");

        $statement = $this->db->prepare("INSERT INTO {$migrations}
            (schema_version, schema_name, applied_at, migration_provenance)
            SELECT :version, :schema, :applied, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$migrations} WHERE schema_version = :existing_version)");
        $statement->execute([
            ':version' => self::VERSION,
            ':schema' => self::SCHEMA,
            ':applied' => $appliedAt,
            ':provenance' => $encoded,
            ':existing_version' => self::VERSION,
        ]);
    }

    /** Rollback is preservation-only: state, journal, denials, legacy registry, and recovery registry are never deleted. */
    public function preserveForRollback(string $occurredAt, array $provenance): array
    {
        self::assertTimestamp($occurredAt);
        $encoded = self::encodeCanonical($provenance);
        if ($provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $events = $this->table('wpuiai_cutover_schema_events');
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
        if ($nullable && $timestamp === null) {
            return;
        }
        if (!is_string($timestamp) || preg_match('/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/', $timestamp) !== 1) {
            throw new InvalidArgumentException('timestamp must be ISO-8601 UTC');
        }
    }

    public static function encodeCanonical(array $value): string
    {
        ksort($value, SORT_STRING);
        return json_encode($value, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);
    }
}

final class FocusaSpec152eAuthorityCutoverService
{
    public const RESULT_SCHEMA = 'focusa.spec152e.authority_cutover_result.v1';
    public const VERSION = 1;

    public const CUTOVER_STATE_KEY = 'cutover_v1';
    public const AUTHORITY = 'WPUIAI.com EDD';
    public const NEW_ISSUANCE = 'edd_authority_only';
    public const FACADE_ROLE = 'presenter_and_bounded_proxy_only';
    public const SPEC158 = 'excluded';
    public const ISSUANCE_REQUIREMENTS = [
        'verified_registration' => true,
        'edd_order_bound' => true,
        'no_local_issuance' => true,
    ];

    /** Server-owned denied issuance surfaces and their exact fail-closed codes (Spec 152E §2.11, §12, §22.3 step 8). */
    public const DENIED_SURFACES = [
        'install_site_create' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'install_site_payment' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'install_site_webhook' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'wpuiai_custom_issue' => ['code' => 'INSTALL_SITE_ISSUANCE_DISABLED', 'next_action' => 'use_edd_authority_checkout'],
        'stripe_direct_product' => ['code' => 'STRIPE_DIRECT_FLOW_DENIED', 'next_action' => 'use_edd_checkout'],
        'local_self_eval' => ['code' => 'LOCAL_EVALUATION_DENIED', 'next_action' => 'use_edd_evaluation'],
    ];

    /** Legacy install-site tables that become read-only after cutover (Spec 152E §22.3 step 10). */
    public const LEGACY_TABLES = [
        'wpuiai_licenses' => ['classification' => 'noncanonical_license_registry', 'migration' => 'evidence_inventory_migrate_or_quarantine_then_read_only'],
        'wpuiai_license_audit' => ['classification' => 'noncanonical_audit_evidence', 'migration' => 'preserve_for_reconciliation'],
    ];

    /** The only legacy-table operation retained after cutover: bounded validation/recovery reads. */
    public const LEGACY_ALLOWED_OPERATIONS = ['SELECT'];

    /** Bounded retained legacy validation/recovery surfaces (Spec 152E §18, §22.3 step 9). None grant entitlement. */
    public const RETAINED_SURFACES = [
        'legacy_validate' => ['route' => '/wpuiai-ai-cloud/v1/license/validate', 'retained_for' => 'validation'],
        'legacy_keys_validate' => ['route' => '/wpuiai-ai-cloud/v1/keys/validate', 'retained_for' => 'validation'],
        'legacy_status' => ['route' => '/wpuiai-ai-cloud/v1/license/status', 'retained_for' => 'recovery'],
        'recovery_status' => ['route' => '/v1/recovery/status', 'retained_for' => 'recovery'],
        'recovery_export' => ['route' => '/v1/recovery/export', 'retained_for' => 'recovery'],
        'recovery_diagnostics' => ['route' => '/v1/recovery/diagnostics', 'retained_for' => 'recovery'],
        'recovery_repair' => ['route' => '/v1/recovery/repair', 'retained_for' => 'recovery'],
        'recovery_update' => ['route' => '/v1/recovery/update', 'retained_for' => 'recovery'],
        'recovery_uninstall' => ['route' => '/v1/recovery/uninstall', 'retained_for' => 'recovery'],
    ];
    public const RETAINED_FOR = ['validation', 'recovery'];

    /** Registered facade activation surfaces: every one is an EDD authority proxy, never a local issuance route. */
    public const FACADE_ACTIONS = [
        'activation_start', 'activation_verify', 'activation_offers',
        'activation_select_offer', 'activation_checkout',
        'activation_existing_license', 'activation_poll', 'lease_refresh',
        'nodes_list', 'nodes_deactivate', 'account_manage_link',
    ];

    /** Install-site legacy routes that proxy to the authority instead of acting locally. */
    public const INSTALL_SITE_PROXY_ACTIONS = ['license_activate', 'license_deactivate'];

    private const GENESIS_DIGEST = '0000000000000000000000000000000000000000000000000000000000000000';
    private const REQUEST_PATTERN = '/^req_[a-z0-9_]{4,64}$/D';
    private const IDEMPOTENCY_PATTERN = '/^idem_[a-z0-9_]{4,64}$/D';
    private const DIGEST_PATTERN = '/^[0-9a-f]{64}$/D';
    private const ROUTE_PATTERN = '#^/(?:v1/|wpuiai-ai-cloud/v1/)[A-Za-z0-9_./-]+$#D';
    private const STATE_VERSION_PATTERN = '/^[A-Za-z0-9._-]{1,64}$/D';

    /** @var Closure(): string */
    private Closure $clock;

    public function __construct(
        private PDO $db,
        private FocusaSpec152eAuthorityCutoverSchema $schema,
        callable $clock,
    ) {
        $this->clock = Closure::fromCallable($clock);
        $this->db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
    }

    /**
     * Publish the exact cutover state. Idempotent and immutable: a replay with
     * the same payload returns the stored state (replayed=true, zero writes);
     * any different payload fails closed with CUTOVER_STATE_ALREADY_PUBLISHED.
     * Every registry is validated against server-owned allowlists; denial codes
     * and next actions come from this contract, never from the caller.
     */
    public function publishCutoverState(array $input): array
    {
        $version = (string) ($input['cutover_version'] ?? '');
        $effectiveAt = (string) ($input['effective_at'] ?? '');
        if (preg_match(self::STATE_VERSION_PATTERN, $version) !== 1) {
            throw new InvalidArgumentException('bounded cutover version required');
        }
        FocusaSpec152eAuthorityCutoverSchema::assertTimestamp($effectiveAt);
        $this->assertCorrelation($input);
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }

        $denied = $this->validateDeniedSurfaces($input['denied_issuance_surfaces'] ?? []);
        $legacy = $this->validateLegacyTables($input['legacy_read_only_tables'] ?? []);
        $recovery = $this->validateRecoverySurfaces($input['retained_recovery_surfaces'] ?? []);
        $facade = $this->validateFacadeProxyRoutes($input['facade_proxy_routes'] ?? [], $input['edd_authority_endpoints'] ?? []);
        $installProxy = $this->validateInstallSiteProxyRoutes($input['install_site_proxy_routes'] ?? []);
        $legacyReadRoutes = $this->validateLegacyReadRoutes($input['legacy_read_only_routes'] ?? [], $recovery);

        $payload = [
            'cutover_version' => $version,
            'effective_at' => $effectiveAt,
            'authority' => self::AUTHORITY,
            'new_issuance' => self::NEW_ISSUANCE,
            'facade_role' => self::FACADE_ROLE,
            'spec158' => self::SPEC158,
            'issuance_requirements' => self::ISSUANCE_REQUIREMENTS,
            'denied_issuance_surfaces' => $denied,
            'legacy_read_only_tables' => $legacy,
            'retained_recovery_surfaces' => $recovery,
            'facade_proxy_routes' => $facade,
            'edd_authority_endpoints' => $input['edd_authority_endpoints'],
            'install_site_proxy_routes' => $installProxy,
            'legacy_read_only_routes' => $legacyReadRoutes,
        ];
        $digest = hash('sha256', FocusaSpec152eAuthorityCutoverSchema::encodeCanonical($payload));

        $stateTable = $this->schema->table('wpuiai_cutover_state');
        $existingStmt = $this->db->prepare("SELECT state_payload, state_digest, published_at FROM {$stateTable} WHERE state_key = :key");
        $existingStmt->execute([':key' => self::CUTOVER_STATE_KEY]);
        $existing = $existingStmt->fetch(PDO::FETCH_ASSOC);
        if ($existing !== false) {
            if (!hash_equals((string) $existing['state_digest'], $digest)) {
                throw new DomainException('CUTOVER_STATE_ALREADY_PUBLISHED');
            }
            return $this->stateEnvelope(json_decode((string) $existing['state_payload'], true), $digest, (string) $existing['published_at'], true);
        }

        $now = ($this->clock)();
        FocusaSpec152eAuthorityCutoverSchema::assertTimestamp($now);
        $stateInsert = $this->db->prepare("INSERT INTO {$stateTable}
            (state_key, cutover_version, effective_at, state_payload, state_digest, published_at, migration_provenance)
            VALUES (:key, :version, :effective, :payload, :digest, :published, :provenance)");
        $stateInsert->execute([
            ':key' => self::CUTOVER_STATE_KEY,
            ':version' => $version,
            ':effective' => $effectiveAt,
            ':payload' => FocusaSpec152eAuthorityCutoverSchema::encodeCanonical($payload),
            ':digest' => $digest,
            ':published' => $now,
            ':provenance' => FocusaSpec152eAuthorityCutoverSchema::encodeCanonical($provenance),
        ]);

        $legacyInsert = $this->db->prepare("INSERT INTO {$this->schema->table('wpuiai_cutover_legacy_tables')}
            (table_name, read_only, allow_operations, classification, migration) VALUES (:t, 'true', :ops, :class, :mig)");
        foreach ($legacy as $row) {
            $spec = self::LEGACY_TABLES[$row['table']];
            $legacyInsert->execute([
                ':t' => $row['table'],
                ':ops' => json_encode(self::LEGACY_ALLOWED_OPERATIONS, JSON_THROW_ON_ERROR),
                ':class' => $spec['classification'],
                ':mig' => $spec['migration'],
            ]);
        }
        $recoveryInsert = $this->db->prepare("INSERT INTO {$this->schema->table('wpuiai_cutover_recovery_surfaces')}
            (surface, route, retained_for, grants_entitlement) VALUES (:s, :r, :for, 'false')");
        foreach ($recovery as $row) {
            $recoveryInsert->execute([':s' => $row['surface'], ':r' => $row['route'], ':for' => $row['retained_for']]);
        }

        $this->journalEvent('cutover_published', self::CUTOVER_STATE_KEY, $now, ['state_digest' => $digest], $provenance, hash('sha256', 'cutover_published' . "\n" . $digest));

        return $this->stateEnvelope($payload, $digest, $now, false);
    }

    /** The published cutover state, or null before publish. Null forces every gate to fail closed. */
    public function cutoverState(): ?array
    {
        $stateTable = $this->schema->table('wpuiai_cutover_state');
        $stmt = $this->db->prepare("SELECT state_payload, state_digest, published_at FROM {$stateTable} WHERE state_key = :key");
        $stmt->execute([':key' => self::CUTOVER_STATE_KEY]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return null;
        }
        return $this->stateEnvelope(json_decode((string) $row['state_payload'], true), (string) $row['state_digest'], (string) $row['published_at'], false);
    }

    /**
     * Recompute the published state digest from the stored payload. The digest
     * covers every published registry, so any drift is immediately visible.
     */
    public function stateDigest(): string
    {
        $stateTable = $this->schema->table('wpuiai_cutover_state');
        $stmt = $this->db->prepare("SELECT state_payload, state_digest FROM {$stateTable} WHERE state_key = :key");
        $stmt->execute([':key' => self::CUTOVER_STATE_KEY]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        if ($row === false) {
            return '';
        }
        $recomputed = hash('sha256', FocusaSpec152eAuthorityCutoverSchema::encodeCanonical(json_decode((string) $row['state_payload'], true)));
        return hash_equals($recomputed, (string) $row['state_digest']) ? $recomputed : '';
    }

    /**
     * Classify an install-site route after cutover:
     *   denied_issuance   -> create/payment/webhook/custom issue/self-eval routes
     *   proxy_to_authority-> legacy activation routes that now proxy to EDD authority
     *   legacy_read_only  -> bounded validation/recovery reads on legacy state
     * Unknown routes fail closed with FACADE_ROUTE_DENIED; nothing is issued locally.
     */
    public function routeDisposition(string $route, array $input): array
    {
        $state = $this->requireCutoverState();
        $this->assertCorrelation($input);

        foreach ($state['denied_issuance_surfaces'] as $denied) {
            if ($denied['route'] === $route) {
                $spec = self::DENIED_SURFACES[$denied['surface']];
                return [
                    'ok' => true,
                    'schema' => self::RESULT_SCHEMA,
                    'disposition' => 'denied_issuance',
                    'surface' => $denied['surface'],
                    'route' => $route,
                    'denial_code' => $spec['code'],
                    'next_action' => $spec['next_action'],
                ];
            }
        }
        foreach ($state['install_site_proxy_routes'] as $proxy) {
            if ($proxy['route'] === $route) {
                return [
                    'ok' => true,
                    'schema' => self::RESULT_SCHEMA,
                    'disposition' => 'proxy_to_authority',
                    'action' => $proxy['action'],
                    'route' => $route,
                    'authority_route' => $proxy['authority_route'],
                ];
            }
        }
        foreach ($state['legacy_read_only_routes'] as $read) {
            if ($read['route'] === $route) {
                return [
                    'ok' => true,
                    'schema' => self::RESULT_SCHEMA,
                    'disposition' => 'legacy_read_only',
                    'surface' => $read['surface'],
                    'route' => $route,
                    'retained_for' => $read['retained_for'],
                    'grants_entitlement' => false,
                ];
            }
        }
        throw new DomainException('FACADE_ROUTE_DENIED');
    }

    /**
     * Deny a new direct install-site issuance attempt (create/payment/webhook
     * or the caller-parameter custom issue route). The attempt is journaled and
     * audited exactly once per (surface, request, idempotency) tuple; replays
     * return the stored denial. Never issues, never writes legacy tables.
     */
    public function denyInstallSiteIssuance(array $input): array
    {
        return $this->denySurface($input, ['install_site_create', 'install_site_payment', 'install_site_webhook', 'wpuiai_custom_issue']);
    }

    /** Deny a direct Stripe product flow attempt (no facade/Stripe metadata may create entitlement truth). */
    public function denyDirectStripeFlow(array $input): array
    {
        return $this->denySurface($input, ['stripe_direct_product']);
    }

    /** Deny a local self-Evaluation (--eval) attempt: Evaluation is EDD-backed authority issuance only. */
    public function denySelfEvaluation(array $input): array
    {
        return $this->denySurface($input, ['local_self_eval']);
    }

    /**
     * Legacy table read-only gate. After cutover the legacy install-site tables
     * accept SELECT (bounded validation/recovery) and reject every mutation
     * with LEGACY_TABLE_READ_ONLY before any statement executes. Unregistered
     * tables fail closed as well: nothing new is written into legacy truth.
     */
    public function legacyTableReadOnlyGate(string $table, string $operation, array $input): array
    {
        $this->requireCutoverState();
        $this->assertCorrelation($input);
        if ($table === '' || preg_match('/^[A-Za-z0-9_]{1,191}$/D', $table) !== 1) {
            throw new DomainException('LEGACY_TABLE_READ_ONLY');
        }
        $legacyTable = $this->schema->table('wpuiai_cutover_legacy_tables');
        $stmt = $this->db->prepare("SELECT read_only, allow_operations FROM {$legacyTable} WHERE table_name = :t");
        $stmt->execute([':t' => $table]);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        if ($row === false || $row['read_only'] !== 'true') {
            throw new DomainException('LEGACY_TABLE_READ_ONLY');
        }
        $allowed = json_decode((string) $row['allow_operations'], true, 512, JSON_THROW_ON_ERROR);
        $normalized = strtoupper($operation);
        if (!in_array($normalized, $allowed, true)) {
            throw new DomainException('LEGACY_TABLE_READ_ONLY');
        }
        return [
            'ok' => true,
            'schema' => self::RESULT_SCHEMA,
            'table' => $table,
            'operation' => $normalized,
            'permitted' => true,
            'reason' => 'bounded_legacy_validation_recovery',
            'grants_entitlement' => false,
        ];
    }

    /** Bounded legacy validation/recovery surface retained after cutover; never grants entitlement. */
    public function retainLegacyValidationRecovery(string $surface): array
    {
        $state = $this->requireCutoverState();
        foreach ($state['retained_recovery_surfaces'] as $retained) {
            if ($retained['surface'] === $surface) {
                return [
                    'ok' => true,
                    'schema' => self::RESULT_SCHEMA,
                    'surface' => $surface,
                    'route' => $retained['route'],
                    'retained_for' => $retained['retained_for'],
                    'grants_entitlement' => false,
                ];
            }
        }
        throw new DomainException('FACADE_ROUTE_DENIED');
    }

    /**
     * Facade proxy gate: every registered facade activation surface resolves to
     * its EDD authority kernel route. There is no local issuance action on the
     * facade; any other action fails closed with FACADE_ROUTE_DENIED.
     */
    public function facadeProxyGate(string $action, array $input): array
    {
        $state = $this->requireCutoverState();
        $this->assertCorrelation($input);
        if (!in_array($action, self::FACADE_ACTIONS, true)) {
            throw new DomainException('FACADE_ROUTE_DENIED');
        }
        if (!isset($state['facade_proxy_routes'][$action])) {
            throw new DomainException('FACADE_ROUTE_DENIED');
        }
        $authorityRoute = $state['facade_proxy_routes'][$action];
        if ($authorityRoute !== ($state['edd_authority_endpoints'][$action] ?? null)) {
            throw new DomainException('FACADE_ROUTE_DENIED');
        }
        return [
            'ok' => true,
            'schema' => self::RESULT_SCHEMA,
            'action' => $action,
            'facade_id' => 'focusa_install_v1',
            'authority_route' => $authorityRoute,
            'issuance' => 'edd_authority_only',
        ];
    }

    /** Recompute the digest-chained cutover journal from genesis. */
    public function journalChainValid(): bool
    {
        $journal = $this->schema->table('wpuiai_cutover_state_journal');
        $rows = $this->db->query("SELECT journal_seq, journal_key, event_type, state_key, occurred_at, detail, previous_digest, entry_digest, migration_provenance FROM {$journal} ORDER BY journal_seq ASC")->fetchAll(PDO::FETCH_ASSOC);
        $previous = self::GENESIS_DIGEST;
        foreach ($rows as $row) {
            if (!hash_equals($previous, (string) $row['previous_digest'])) {
                return false;
            }
            $expected = $this->entryDigest(
                (string) $row['previous_digest'],
                (int) $row['journal_seq'],
                (string) $row['journal_key'],
                (string) $row['event_type'],
                (string) $row['state_key'],
                (string) $row['occurred_at'],
                (string) $row['detail'],
                (string) $row['migration_provenance'],
            );
            if (!hash_equals($expected, (string) $row['entry_digest'])) {
                return false;
            }
            $previous = (string) $row['entry_digest'];
        }
        return true;
    }

    public function countRows(string $table): int
    {
        if (preg_match('/^[A-Za-z0-9_]{1,191}$/D', $table) !== 1) {
            throw new InvalidArgumentException('invalid table name');
        }
        return (int) $this->db->query("SELECT COUNT(*) FROM {$this->schema->table($table)}")->fetchColumn();
    }

    private function denySurface(array $input, array $surfaces): array
    {
        $state = $this->requireCutoverState();
        $this->assertCorrelation($input);
        $provenance = $input['migration_provenance'] ?? [];
        if (!is_array($provenance) || $provenance === []) {
            throw new InvalidArgumentException('migration provenance is required');
        }
        $surface = (string) ($input['surface'] ?? '');
        $route = (string) ($input['route'] ?? '');
        if (!in_array($surface, $surfaces, true) || !isset(self::DENIED_SURFACES[$surface])) {
            throw new DomainException('FACADE_ROUTE_DENIED');
        }
        $spec = self::DENIED_SURFACES[$surface];
        $published = null;
        foreach ($state['denied_issuance_surfaces'] as $denied) {
            if ($denied['surface'] === $surface) {
                $published = $denied;
                break;
            }
        }
        if ($published === null || $published['route'] !== $route) {
            throw new DomainException('FACADE_ROUTE_DENIED');
        }

        $now = ($this->clock)();
        FocusaSpec152eAuthorityCutoverSchema::assertTimestamp($now);
        $denialKey = hash('sha256', $surface . "\n" . (string) $input['request_id'] . "\n" . (string) $input['idempotency_key']);
        $journaled = $this->journalEvent(
            'issuance_denied',
            self::CUTOVER_STATE_KEY,
            $now,
            ['surface' => $surface, 'route' => $route, 'denial_code' => $spec['code'], 'next_action' => $spec['next_action']],
            $provenance,
            $denialKey,
        );
        $denialInsert = $this->db->prepare("INSERT INTO {$this->schema->table('wpuiai_cutover_denials')}
            (denial_uuid, denial_key, surface, route, denial_code, next_action, request_id, idempotency_key, occurred_at, migration_provenance)
            SELECT :uuid, :key, :surface, :route, :code, :next, :request, :idem, :occurred, :provenance
            WHERE NOT EXISTS (SELECT 1 FROM {$this->schema->table('wpuiai_cutover_denials')} WHERE denial_key = :existing)");
        $denialInsert->execute([
            ':uuid' => self::uuid(),
            ':key' => $denialKey,
            ':surface' => $surface,
            ':route' => $route,
            ':code' => $spec['code'],
            ':next' => $spec['next_action'],
            ':request' => (string) $input['request_id'],
            ':idem' => (string) $input['idempotency_key'],
            ':occurred' => $now,
            ':provenance' => FocusaSpec152eAuthorityCutoverSchema::encodeCanonical($provenance),
            ':existing' => $denialKey,
        ]);
        return [
            'ok' => false,
            'schema' => self::RESULT_SCHEMA,
            'denied' => true,
            'surface' => $surface,
            'route' => $route,
            'denial_code' => $spec['code'],
            'next_action' => $spec['next_action'],
            'replayed' => $journaled['replayed'],
            'occurred_at' => $now,
        ];
    }

    private function requireCutoverState(): array
    {
        $state = $this->cutoverState();
        if ($state === null) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        return $state;
    }

    /** Append a digest-chained journal event; replays with the same journal key never append a second entry. */
    private function journalEvent(string $eventType, string $stateKey, string $occurredAt, array $detail, array $provenance, ?string $journalKey = null): array
    {
        FocusaSpec152eAuthorityCutoverSchema::assertTimestamp($occurredAt);
        $encoded = FocusaSpec152eAuthorityCutoverSchema::encodeCanonical($detail);
        if ($journalKey === null) {
            $journalKey = hash('sha256', $eventType . "\n" . $occurredAt . "\n" . $encoded);
        }
        $journal = $this->schema->table('wpuiai_cutover_state_journal');
        $lookup = $this->db->prepare("SELECT journal_seq FROM {$journal} WHERE journal_key = :key");
        $lookup->execute([':key' => $journalKey]);
        $row = $lookup->fetch(PDO::FETCH_ASSOC);
        if ($row !== false) {
            return ['replayed' => true, 'journal_seq' => (int) $row['journal_seq']];
        }
        $seq = (int) $this->db->query("SELECT COALESCE(MAX(journal_seq), 0) + 1 FROM {$journal}")->fetchColumn();
        $previous = self::GENESIS_DIGEST;
        if ($seq > 1) {
            $prevStmt = $this->db->prepare("SELECT entry_digest FROM {$journal} WHERE journal_seq = :prev");
            $prevStmt->execute([':prev' => $seq - 1]);
            $prevRow = $prevStmt->fetch(PDO::FETCH_ASSOC);
            if ($prevRow !== false) {
                $previous = (string) $prevRow['entry_digest'];
            }
        }
        $provenanceEncoded = FocusaSpec152eAuthorityCutoverSchema::encodeCanonical($provenance);
        $entry = $this->entryDigest($previous, $seq, $journalKey, $eventType, $stateKey, $occurredAt, $encoded, $provenanceEncoded);
        $stmt = $this->db->prepare("INSERT INTO {$journal}
            (journal_seq, journal_key, event_type, state_key, occurred_at, detail, previous_digest, entry_digest, migration_provenance)
            VALUES (:seq, :key, :event, :state, :occurred, :detail, :prev, :entry, :provenance)");
        $stmt->execute([
            ':seq' => $seq,
            ':key' => $journalKey,
            ':event' => $eventType,
            ':state' => $stateKey,
            ':occurred' => $occurredAt,
            ':detail' => $encoded,
            ':prev' => $previous,
            ':entry' => $entry,
            ':provenance' => $provenanceEncoded,
        ]);
        return ['replayed' => false, 'journal_seq' => $seq];
    }

    private function entryDigest(string $previous, int $seq, string $journalKey, string $eventType, string $stateKey, string $occurredAt, string $detail, string $provenance): string
    {
        return hash('sha256', $previous . "\n" . $seq . "\n" . $journalKey . "\n" . $eventType . "\n" . $stateKey . "\n" . $occurredAt . "\n" . $detail . "\n" . $provenance);
    }

    private function stateEnvelope(array $payload, string $digest, string $publishedAt, bool $replayed): array
    {
        if (preg_match(self::DIGEST_PATTERN, $digest) !== 1) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        return [
            'ok' => true,
            'schema' => self::RESULT_SCHEMA,
            'state_key' => self::CUTOVER_STATE_KEY,
            'cutover_version' => $payload['cutover_version'],
            'effective_at' => $payload['effective_at'],
            'published_at' => $publishedAt,
            'state_digest' => $digest,
            'replayed' => $replayed,
            'authority' => $payload['authority'],
            'new_issuance' => $payload['new_issuance'],
            'facade_role' => $payload['facade_role'],
            'spec158' => $payload['spec158'],
            'issuance_requirements' => $payload['issuance_requirements'],
            'denied_issuance_surfaces' => $payload['denied_issuance_surfaces'],
            'legacy_read_only_tables' => $payload['legacy_read_only_tables'],
            'retained_recovery_surfaces' => $payload['retained_recovery_surfaces'],
            'facade_proxy_routes' => $payload['facade_proxy_routes'],
            'edd_authority_endpoints' => $payload['edd_authority_endpoints'],
            'install_site_proxy_routes' => $payload['install_site_proxy_routes'],
            'legacy_read_only_routes' => $payload['legacy_read_only_routes'],
        ];
    }

    private function validateDeniedSurfaces(array $denied): array
    {
        $out = [];
        foreach ($denied as $entry) {
            $surface = (string) ($entry['surface'] ?? '');
            $route = (string) ($entry['route'] ?? '');
            if (!isset(self::DENIED_SURFACES[$surface])) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            if (preg_match(self::ROUTE_PATTERN, $route) !== 1 && $route !== 'direct-stripe-product-flow' && $route !== 'local-eval-flag') {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            $out[] = ['surface' => $surface, 'route' => $route];
        }
        if (count($out) !== count(self::DENIED_SURFACES)) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        return $out;
    }

    private function validateLegacyTables(array $tables): array
    {
        $out = [];
        foreach ($tables as $entry) {
            $table = (string) ($entry['table'] ?? '');
            if (!isset(self::LEGACY_TABLES[$table])) {
                throw new DomainException('LEGACY_TABLE_READ_ONLY');
            }
            $out[] = ['table' => $table];
        }
        if (count($out) !== count(self::LEGACY_TABLES)) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        return $out;
    }

    private function validateRecoverySurfaces(array $surfaces): array
    {
        $out = [];
        foreach ($surfaces as $entry) {
            $surface = (string) ($entry['surface'] ?? '');
            if (!isset(self::RETAINED_SURFACES[$surface])) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            $spec = self::RETAINED_SURFACES[$surface];
            $route = (string) ($entry['route'] ?? '');
            $retainedFor = (string) ($entry['retained_for'] ?? '');
            $grants = (bool) ($entry['grants_entitlement'] ?? true);
            if ($route !== $spec['route'] || !in_array($retainedFor, self::RETAINED_FOR, true) || $retainedFor !== $spec['retained_for'] || $grants !== false) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            $out[] = ['surface' => $surface, 'route' => $route, 'retained_for' => $retainedFor];
        }
        if (count($out) !== count(self::RETAINED_SURFACES)) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        return $out;
    }

    private function validateFacadeProxyRoutes(array $facade, array $edd): array
    {
        if (count($facade) !== count(self::FACADE_ACTIONS) || count($edd) !== count(self::FACADE_ACTIONS)) {
            throw new DomainException('FACADE_ROUTE_DENIED');
        }
        $out = [];
        foreach (self::FACADE_ACTIONS as $action) {
            $facadeRoute = (string) ($facade[$action] ?? '');
            $eddRoute = (string) ($edd[$action] ?? '');
            if (preg_match(self::ROUTE_PATTERN, $facadeRoute) !== 1 || preg_match(self::ROUTE_PATTERN, $eddRoute) !== 1) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            if ($facadeRoute !== $eddRoute) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            $out[$action] = $facadeRoute;
        }
        return $out;
    }

    private function validateInstallSiteProxyRoutes(array $proxy): array
    {
        if (count($proxy) !== count(self::INSTALL_SITE_PROXY_ACTIONS)) {
            throw new DomainException('FACADE_ROUTE_DENIED');
        }
        $out = [];
        foreach (self::INSTALL_SITE_PROXY_ACTIONS as $action) {
            $entry = null;
            foreach ($proxy as $candidate) {
                if ((string) ($candidate['action'] ?? '') === $action) {
                    $entry = $candidate;
                    break;
                }
            }
            if ($entry === null) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            $route = (string) ($entry['route'] ?? '');
            $authorityRoute = (string) ($entry['authority_route'] ?? '');
            if (preg_match(self::ROUTE_PATTERN, $route) !== 1 || strpos($route, '/wpuiai-ai-cloud/v1/') !== 0
                || preg_match(self::ROUTE_PATTERN, $authorityRoute) !== 1 || strpos($authorityRoute, '/v1/') !== 0) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            $out[] = ['action' => $action, 'route' => $route, 'authority_route' => $authorityRoute];
        }
        return $out;
    }

    private function validateLegacyReadRoutes(array $routes, array $recovery): array
    {
        $recoveryBySurface = [];
        foreach ($recovery as $row) {
            $recoveryBySurface[$row['surface']] = $row;
        }
        $out = [];
        foreach ($routes as $entry) {
            $route = (string) ($entry['route'] ?? '');
            $surface = (string) ($entry['surface'] ?? '');
            if (!isset($recoveryBySurface[$surface]) || $recoveryBySurface[$surface]['route'] !== $route) {
                throw new DomainException('FACADE_ROUTE_DENIED');
            }
            $out[] = ['route' => $route, 'surface' => $surface, 'retained_for' => $recoveryBySurface[$surface]['retained_for']];
        }
        if (count($out) !== 3) {
            throw new DomainException('CUTOVER_STATE_REQUIRED');
        }
        return $out;
    }

    private function assertCorrelation(array $input): void
    {
        if (preg_match(self::REQUEST_PATTERN, (string) ($input['request_id'] ?? '')) !== 1) {
            throw new InvalidArgumentException('request_id required');
        }
        if (preg_match(self::IDEMPOTENCY_PATTERN, (string) ($input['idempotency_key'] ?? '')) !== 1) {
            throw new InvalidArgumentException('idempotency_key required');
        }
    }

    private static function uuid(): string
    {
        $bytes = random_bytes(16);
        $bytes[6] = chr((ord($bytes[6]) & 0x0f) | 0x40);
        $bytes[8] = chr((ord($bytes[8]) & 0x3f) | 0x80);
        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($bytes), 4));
    }
}
