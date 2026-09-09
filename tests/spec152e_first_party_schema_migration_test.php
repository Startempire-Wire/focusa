<?php
declare(strict_types=1);
require_once __DIR__ . '/../docs/contracts/spec152e-edd-bound-lease-issuer.v1.php';

// Independent metadata-boundary tests; not a substitute for MySQL engine proof.
final class FirstPartySchemaStatement extends PDOStatement {
    public function __construct(private array $rows) {}
    public function execute(?array $params = null): bool { return true; }
    public function fetchAll(int $mode = PDO::FETCH_DEFAULT, mixed ...$args): array { return $this->rows; }
}
final class FirstPartySchemaDatabase extends PDO {
    public array $writes = [];
    public string $engine = 'InnoDB';
    public bool $missingTable = false;
    public function __construct(public array $columns, public array $checks, private string $version) { parent::__construct('sqlite::memory:'); }
    public function getAttribute(int $attribute): mixed { return $attribute === PDO::ATTR_DRIVER_NAME ? 'mysql' : $this->version; }
    public function prepare(string $query, array $options = []): PDOStatement|false {
        if (str_contains($query, 'information_schema.TABLES')) {
            $names = ['leases', 'lease_sequences', 'lease_idempotency', 'lease_schema_migrations', 'lease_schema_events', 'accounts', 'nodes'];
            if ($this->missingTable) array_pop($names);
            return new FirstPartySchemaStatement(array_map(fn($name) => ['TABLE_NAME' => 'wp_wpuiai_authority_' . $name, 'ENGINE' => $this->engine], $names));
        }
        return new FirstPartySchemaStatement(str_contains($query, 'information_schema.COLUMNS') ? $this->columns : $this->checks);
    }
    public function exec(string $statement): int|false { $this->writes[] = $statement; return 0; }
}
function schema_check(bool $ok, string $label): void {
    if (!$ok) throw new RuntimeException($label);
}
function run_schema_case(array $columns, array $checks, string $version = '8.0.44', ?string $error = null): array {
    $db = new FirstPartySchemaDatabase($columns, $checks, $version);
    $keys = new FocusaSpec152eAuthorityKeySetSeam(str_repeat("\x01", 32), str_repeat("\x02", 32), static fn() => '2026-09-07T00:00:00Z');
    $issuer = new FocusaSpec152eEddBoundLeaseIssuer($db, $keys, static fn() => '2026-09-07T00:00:00Z');
    try {
        (new ReflectionMethod($issuer, 'upgradeFirstPartyLeaseSchema'))->invoke($issuer, 'wp_wpuiai_authority_leases');
        schema_check($error === null, 'expected migration rejection');
    } catch (DomainException $failure) {
        schema_check($failure->getMessage() === $error, 'unexpected migration rejection: ' . $failure->getMessage());
        schema_check($db->writes === [], 'rejection must precede every schema write');
    }
    return $db->writes;
}
function run_first_party_schema_boundary_tests(): array {
$columns = array_map(static fn($name) => ['COLUMN_NAME' => $name, 'COLUMN_TYPE' => 'bigint(20) unsigned', 'IS_NULLABLE' => 'NO'], ['edd_order_id', 'edd_order_item_id', 'edd_license_id']);
$old = [['CONSTRAINT_NAME' => 'lease_posture', 'CHECK_CLAUSE' => "(`posture` in (_utf8mb4'paid',_utf8mb4'evaluation',_utf8mb4'bundle'))"]];
foreach (['8.0.44' => 'DROP CONSTRAINT', '10.11.13-MariaDB' => 'DROP CHECK'] as $version => $drop) {
    $writes = run_schema_case($columns, $old, $version);
    schema_check(count($writes) === 1 && str_contains($writes[0], $drop . ' `lease_posture`'), 'one engine-correct alteration');
    schema_check(substr_count($writes[0], 'MODIFY COLUMN') === 3 && str_contains($writes[0], 'edd_license_id IS NOT NULL'), 'nullable provenance retains nondeveloper billing enforcement');
}
$new = [
    ['CONSTRAINT_NAME' => 'lease_posture', 'CHECK_CLAUSE' => "posture IN ('paid','evaluation','bundle','developer')"],
    ['CONSTRAINT_NAME' => 'lease_billing', 'CHECK_CLAUSE' => "posture='developer' OR (edd_order_id IS NOT NULL AND edd_order_item_id IS NOT NULL AND edd_license_id IS NOT NULL)"],
];
$nullable = array_map(static fn($row) => array_replace($row, ['IS_NULLABLE' => 'YES']), $columns);
schema_check(run_schema_case($nullable, $new) === [], 'current schema migration is idempotent');
$unsafe = $old;
$unsafe[0]['CHECK_CLAUSE'] .= ' AND operator_seats > 0';
run_schema_case($columns, $unsafe, '8.0.44', 'LEASE_SCHEMA_POSTURE_CHECK_MISMATCH');
$weak = $new;
$weak[1]['CHECK_CLAUSE'] = str_replace(' AND ', ' OR ', $weak[1]['CHECK_CLAUSE']);
run_schema_case($columns, $weak, '8.0.44', 'LEASE_SCHEMA_BILLING_CHECK_MISMATCH');
$bad = $columns;
$bad[0]['COLUMN_TYPE'] = 'varchar(64)';
run_schema_case($bad, $old, '8.0.44', 'LEASE_SCHEMA_COLUMN_TYPE_MISMATCH');
foreach ([['InnoDB', false, null], ['MyISAM', false, 'AUTHORITY_STORAGE_NOT_TRANSACTIONAL'], ['InnoDB', true, 'AUTHORITY_STORAGE_TABLES_MISSING']] as [$engine, $missing, $error]) {
    $db = new FirstPartySchemaDatabase($columns, $old, '8.0.44');
    $db->engine = $engine;
    $db->missingTable = $missing;
    $keys = new FocusaSpec152eAuthorityKeySetSeam(str_repeat("\x01",32), str_repeat("\x02",32), static fn() => '2026-09-07T00:00:00Z');
    $issuer = new FocusaSpec152eEddBoundLeaseIssuer($db, $keys, static fn() => '2026-09-07T00:00:00Z');
    try {
        (new ReflectionMethod($issuer, 'requireTransactionalStorage'))->invoke($issuer);
        schema_check($error === null, 'nontransactional issuer storage must be rejected');
    } catch (DomainException $failure) {
        schema_check($failure->getMessage() === $error, 'storage gate reports the exact failure');
    }
    schema_check($db->writes === [], 'storage checks never mutate authority tables');
}
return ['schema' => 'focusa.first_party_schema_boundary_validation.v1', 'cases' => 9, 'result' => 'passed', 'mysql_engine_proof' => false];
}
if (realpath($_SERVER['SCRIPT_FILENAME'] ?? '') === __FILE__) {
    echo json_encode(run_first_party_schema_boundary_tests(), JSON_PRETTY_PRINT), PHP_EOL;
}
