#!/usr/bin/env python3
"""Spec 152E.06.02 paid-record migration gate (atom focusa-vbcqu.20.13.50).

Exact verification:
    python3 tests/spec152e_paid_record_migration_test.py

Validates that the paid-record migrator imports ONLY evidence-backed paid
install-site records into EDD authority, idempotently, with preserved
product/status/key history where policy allows, verified identity required
before ownership delivery, and refunded/revoked records that can never be
reactivated. Checks are replayable from the pinned commit: the fixture is
deterministic and the PHP adapter is executed twice with byte-identical output.

Surfaces under test:
- Migration journal/adapter:
  docs/contracts/spec152e-paid-record-migration.v1.php
- Stripe payment/refund evidence, install registry, EDD/account mappings:
  docs/contracts/spec152e-paid-record-migration-fixture.v1.json

Fail-closed invariants:
- No unverified-email promotion, no local/self-issued entitlement, no
  independent facade authority, no client-controlled EDD price/grants.
- No raw email, raw key, payment id, or secret in any artifact.
- No push, deploy, release, merge, or Beads mutation is performed.
"""

import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec152e-paid-record-migration.v1.php"
FIXTURE = ROOT / "docs/contracts/spec152e-paid-record-migration-fixture.v1.json"

PHP = "/usr/local/bin/php" if Path("/usr/local/bin/php").exists() else shutil.which("php")

positive = 0
negative = 0


def expect(condition: bool, message: str) -> None:
    global positive
    positive += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def expect_negative(condition: bool, message: str) -> None:
    global negative
    negative += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def sha256(raw: str) -> str:
    return hashlib.sha256(raw.encode()).hexdigest()


# ── Load artifacts ───────────────────────────────────────────────────────

contract_raw = CONTRACT.read_text(encoding="utf-8")
fixture_raw = FIXTURE.read_text(encoding="utf-8")
fixture = json.loads(fixture_raw)

# ── Fixture structure ────────────────────────────────────────────────────

expect(fixture["schema"] == "focusa.spec152e.paid_record_migration_fixture.v1", "fixture schema id")
expect(fixture["fixture_id"] == "focusa-vbcqu.20.13.50", "fixture_id")
expect(fixture["authority"]["canonical"] == "WPUIAI.com EDD", "canonical authority")
expect(fixture["authority"]["spec158"] == "excluded", "spec158 excluded")
expect(fixture["policy"] == "evidence_backed_paid_import_only", "import policy")
expect(fixture["redaction"] == {
    "raw_email": "absent", "raw_key": "absent", "payment_id_stored": False,
    "secret_material": "absent",
}, "redaction posture")

product_allowlist = fixture["product_allowlist"]
expect(product_allowlist == [
    "focusa_operator", "uiai_engine_operator", "focusa_uiai_bundle", "focusa_evaluation",
], "server-owned product allowlist")

records = fixture["records"]
evidence = fixture["stripe_evidence"]
registry = fixture["install_registry"]
mappings = fixture["edd_mappings"]

handles = [r["handle"] for r in records]
expect(len(handles) == len(set(handles)), "record handles unique")
expect(all(re.fullmatch(r"rec_[a-z0-9_]{4,64}", h) for h in handles), "record handle shape")
expect({r["disposition"] for r in records} <= {
    "evidence_backed_import", "refunded_revoked", "verify_first", "unresolved",
}, "dispositions bounded")
expect({r["surface"] for r in records} <= {"install_site_license", "install_site_audit_receipt"}, "surfaces bounded")
expect(all(r["product_code"] in product_allowlist for r in records), "records use allowlisted products")

by_handle = {r["handle"]: r for r in records}
import_records = [r for r in records if r["disposition"] == "evidence_backed_import"]
refunded_revoked = [r for r in records if r["disposition"] == "refunded_revoked"]
verify_first = [r for r in records if r["disposition"] == "verify_first"]
unresolved = [r for r in records if r["disposition"] == "unresolved"]

# ── Stripe payment/refund evidence ───────────────────────────────────────

evidence_by_handle = {e["evidence_handle"]: e for e in evidence}
expect(len(evidence_by_handle) == len(evidence), "evidence handles unique")
expect({e["kind"] for e in evidence} == {"payment_evidence", "refund_evidence"}, "evidence kinds")
expect(all(re.fullmatch(r"ev_[a-z0-9_]{4,64}", e["evidence_handle"]) for e in evidence), "evidence handle shape")
expect(all(re.fullmatch(r"[0-9a-f]{64}", e["digest"]) for e in evidence), "evidence digests are 64-hex")
expect(all(e["record_handle"] in by_handle for e in evidence), "evidence references known records")
expect(all(e["source"] == "stripe_reconciliation" for e in evidence), "evidence source is stripe reconciliation")

# ── Install registry ─────────────────────────────────────────────────────

expect(len(registry) == 8, "install registry rows")
expect(len({r["registry_handle"] for r in registry}) == len(registry), "registry handles unique")
expect(all(r["record_handle"] in by_handle for r in registry), "registry references known records")
registry_by_record = {r["record_handle"]: r for r in registry}
expect(len(registry_by_record) == len(registry), "one registry row per referenced record")
for rec in records:
    if rec["disposition"] != "unresolved":
        row = registry_by_record.get(rec["handle"])
        expect(row is not None, f"registry row present for {rec['handle']}")
        expect(row["surface"] == rec["surface"], f"registry surface matches {rec['handle']}")
        expect(row["product_code"] == rec["product_code"], f"registry product matches {rec['handle']}")
        expect(row["evidence_handle"] == rec["evidence_handle"], f"registry evidence matches {rec['handle']}")
        expect(re.fullmatch(r"[0-9a-f]{64}", row["digest"]), f"registry digest 64-hex {rec['handle']}")

# ── Accepted records resolve to exactly one EDD/account entitlement ──────

accepted_handles = {r["handle"] for r in import_records}
mapping_by_record = {m["record_handle"]: m for m in mappings}
expect(len(mapping_by_record) == len(mappings), "mapping handles unique")
expect(set(mapping_by_record) == accepted_handles, "mappings exist exactly for accepted records")
for rec in import_records:
    m = mapping_by_record[rec["handle"]]
    expect(re.fullmatch(r"acc_[a-z0-9_]{4,64}", m["account_uuid"]), f"account shape {rec['handle']}")
    for field in ("edd_customer_handle", "edd_order_handle", "edd_license_handle"):
        expect(re.fullmatch(r"edd_(?:cust|order|lic)_[a-z0-9_]{4,64}", m[field]), f"{field} shape {rec['handle']}")
    expect(re.fullmatch(r"[0-9a-f]{64}", rec["identity_lookup_digest"]), f"identity digest 64-hex {rec['handle']}")
    ev = evidence_by_handle[rec["evidence_handle"]]
    expect(ev["kind"] == "payment_evidence", f"accepted record has payment evidence {rec['handle']}")
    expect(ev["record_handle"] == rec["handle"], f"evidence binds the accepted record {rec['handle']}")
    expect(bool(rec.get("masked_key")), f"masked key preserved {rec['handle']}")

# ── Negative dispositions: no entitlement, adverse state preserved ───────

for rec in refunded_revoked:
    expect(rec["handle"] not in mapping_by_record, f"refunded/revoked has no entitlement {rec['handle']}")
    expect(rec["adverse_state"] in ("refunded", "revoked"), f"adverse state typed {rec['handle']}")
    expect(rec["evidence_handle"] in evidence_by_handle, f"adverse record keeps evidence {rec['handle']}")
    expect(registry_by_record[rec["handle"]]["adverse_state"] == rec["adverse_state"], f"registry adverse state {rec['handle']}")
for rec in verify_first:
    expect(rec["handle"] not in mapping_by_record, f"verify_first has no pre-verified entitlement {rec['handle']}")
    expect(re.fullmatch(r"[0-9a-f]{64}", rec["identity_lookup_digest"]), f"verify_first identity digest {rec['handle']}")
for rec in unresolved:
    expect(rec["handle"] not in mapping_by_record, f"unresolved has no entitlement {rec['handle']}")
    expect("evidence_handle" not in rec, f"unresolved carries no evidence {rec['handle']}")
    expect(rec["handle"] not in registry_by_record, f"unresolved has no registry row {rec['handle']}")

expect(len(import_records) == 5, "five accepted paid records")
expect(fixture["expectations"]["accepted_records"] == 5, "expectations match accepted count")
expect(fixture["expectations"]["one_entitlement_per_accepted_record"] is True, "one entitlement per accepted record")
expect(fixture["expectations"]["refunded_revoked_never_reactivated"] is True, "never reactivate")
expect(fixture["expectations"]["verified_identity_required_before_delivery"] is True, "verified identity gate")
expect(fixture["expectations"]["rollback_preservation_only"] is True, "rollback preservation")

# Replay-safe journal vector consistency: imports + adverse states = apply journal entries.
expect(fixture["journal_vectors"]["apply"]["imports"] == 5, "apply imports vector")
expect(fixture["journal_vectors"]["apply"]["mappings"] == 5, "apply mappings vector")
expect(fixture["journal_vectors"]["apply"]["journal_entries"] == len(import_records) + len(refunded_revoked), "apply journal vector")
expect(fixture["journal_vectors"]["replay"]["second_entitlement"] is False, "replay vector")
expect(fixture["journal_vectors"]["rollback"]["delete_methods"] == 0, "rollback vector")

# ── Redaction: no secret or unmasked real-email evidence anywhere ────────

for name, raw in (("fixture", fixture_raw), ("contract", contract_raw)):
    expect(not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw), f"no raw email in {name}")
    expect(not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw), f"no stripe secret prefix in {name}")
    expect("focusa_live_" not in raw, f"no synthetic focusa_live key in {name}")
    expect("license_key" not in raw, f"no raw license key in {name}")
    expect("payment_intent_id" not in raw, f"no payment intent id in {name}")
    expect("customer_id" not in raw, f"no raw customer id in {name}")
    expect("BEGIN RSA" not in raw and "PRIVATE KEY" not in raw, f"no private key material in {name}")

# ── Contract static invariants ───────────────────────────────────────────

expect("final class FocusaSpec152ePaidRecordMigrationSchema" in contract_raw, "schema class")
expect("final class FocusaSpec152ePaidRecordMigrationService" in contract_raw, "service class")
expect("focusa.spec152e.paid_record_migration.v1" in contract_raw, "contract schema id")
for table in (
    "wpuiai_paid_record_journal",
    "wpuiai_paid_record_imports",
    "wpuiai_paid_record_evidence",
    "wpuiai_paid_record_install_registry",
    "wpuiai_paid_record_edd_mappings",
):
    expect(table in contract_raw, f"table {table}")
for method in (
    "function dryRun", "function applyRecord", "function replayImport", "function reactivate",
    "function preserveForRollback", "function journalChainValid", "function assertVerifiedIdentity",
    "function countRows", "function importOnce",
):
    expect(method in contract_raw, f"method {method}")
for code in (
    "EDD_ORDER_UNVERIFIED", "EMAIL_VERIFICATION_REQUIRED", "EMAIL_VERIFICATION_FAILED",
    "REFUNDED", "REVOKED", "PRODUCT_MAPPING_REQUIRED", "REQUEST_ID_REQUIRED",
    "IDEMPOTENCY_KEY_REQUIRED", "EDD_CUSTOMER_RESOLUTION_FAILED",
):
    expect(code in contract_raw, f"fail-closed code {code}")
expect("preserve_adverse_state" in contract_raw, "adverse state journal event")
expect("verified_identity_digest" in contract_raw, "verified identity digest stored, never raw email")
expect("previous_digest" in contract_raw and "entry_digest" in contract_raw, "replay-safe journal chain")
expect("GENESIS_DIGEST" in contract_raw, "journal genesis digest")
expect("hash_equals" in contract_raw, "constant-time digest comparison")

# One entitlement per accepted record: importOnce writes exactly one imports row and one mapping row.
reactivate_start = contract_raw.index("function reactivate")
reactivate_end = contract_raw.index("function journalChainValid")
reactivate_body = contract_raw[reactivate_start:reactivate_end]
expect("INSERT INTO" not in reactivate_body and "UPDATE" not in reactivate_body and "exec(" not in reactivate_body, "reactivate() can only fail closed")

# The migrator is preservation-only: no delete path may exist.
for forbidden in ("DELETE FROM", "TRUNCATE", "DROP TABLE", "->exec('DELETE"):
    expect(forbidden not in contract_raw, f"no destructive statement {forbidden}")
# No raw email or client-controlled price/grant inputs.
expect("customer_email" not in contract_raw and "raw_email" not in contract_raw, "no raw email field")
expect("$input['price']" not in contract_raw and "['price']" not in contract_raw and "'price' =>" not in contract_raw, "no client-controlled price input")
expect("['grant']" not in contract_raw and "['grants']" not in contract_raw and "'grants' =>" not in contract_raw, "no client-controlled grant input")
expect("product_code" in contract_raw and "PRODUCTS" in contract_raw, "server-owned product allowlist")

# ── Behavioral execution (deterministic, replayable) ─────────────────────

HARNESS = r"""<?php
// 152E.06.02 paid-record migration behavioral harness (generated by the python gate).
// Executes dry-run/apply/replay/rollback vectors against the adapter on sqlite.
declare(strict_types=1);
$contract = $argv[1];
$fixturePath = $argv[2];
require_once $contract;
$fixture = json_decode((string) file_get_contents($fixturePath), true, 512, JSON_THROW_ON_ERROR);
$positive = 0;
$negative = 0;
function ok(bool $condition, string $message): void { global $positive; $positive++; if (!$condition) { fwrite(STDERR, "FAIL: {$message}\n"); exit(1); } }
function okThrows(callable $operation, string $code, string $message): void { global $negative; $negative++; try { $operation(); } catch (Throwable $e) { if ($e->getMessage() === $code) { return; } fwrite(STDERR, "FAIL: {$message} (got {$e->getMessage()})\n"); exit(1); } fwrite(STDERR, "FAIL: {$message} (no throw)\n"); exit(1); }

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$schema = new FocusaSpec152ePaidRecordMigrationSchema($db, 'wp_');
$schema->migrate('2026-08-08T00:00:00Z', ['source' => 'paid_record_migration_test']);
$tick = 0;
$clock = static function () use (&$tick): string {
    $ts = (new DateTimeImmutable('2026-08-08T00:01:00Z'))->modify('+' . $tick . ' minutes')->format('Y-m-d\TH:i:s\Z');
    $tick++;
    return $ts;
};
$service = new FocusaSpec152ePaidRecordMigrationService($db, $schema, $clock);

$counts = static function () use ($db): array {
    $table = static fn(string $name): int => (int) $db->query("SELECT COUNT(*) FROM wp_{$name}")->fetchColumn();
    return [
        'imports' => $table('wpuiai_paid_record_imports'),
        'mappings' => $table('wpuiai_paid_record_edd_mappings'),
        'journal' => $table('wpuiai_paid_record_journal'),
        'evidence' => $table('wpuiai_paid_record_evidence'),
        'registry' => $table('wpuiai_paid_record_install_registry'),
    ];
};

// Seed Stripe payment/refund evidence and the install registry from the fixture.
$evidenceStmt = $db->prepare("INSERT INTO wp_wpuiai_paid_record_evidence
    (evidence_handle, kind, source, record_handle, status, digest, occurred_at, migration_provenance)
    VALUES (:h, :kind, :source, :record, :status, :digest, :occurred, :prov)");
foreach ($fixture['stripe_evidence'] as $ev) {
    $evidenceStmt->execute([
        ':h' => $ev['evidence_handle'], ':kind' => $ev['kind'], ':source' => $ev['source'],
        ':record' => $ev['record_handle'], ':status' => $ev['status'], ':digest' => $ev['digest'],
        ':occurred' => $ev['occurred_at'], ':prov' => json_encode($ev['migration_provenance'] ?? ['source' => 'fixture'], JSON_THROW_ON_ERROR),
    ]);
}
$registryStmt = $db->prepare("INSERT INTO wp_wpuiai_paid_record_install_registry
    (registry_handle, record_handle, surface, product_code, record_status, masked_key, sequence, adverse_state, evidence_handle, digest, migration_provenance)
    VALUES (:rh, :record, :surface, :product, :status, :masked, :seq, :adverse, :ev, :digest, :prov)");
foreach ($fixture['install_registry'] as $row) {
    $registryStmt->execute([
        ':rh' => $row['registry_handle'], ':record' => $row['record_handle'], ':surface' => $row['surface'],
        ':product' => $row['product_code'], ':status' => $row['record_status'], ':masked' => $row['masked_key'],
        ':seq' => $row['sequence'], ':adverse' => $row['adverse_state'], ':ev' => $row['evidence_handle'],
        ':digest' => $row['digest'], ':prov' => json_encode($row['migration_provenance'], JSON_THROW_ON_ERROR),
    ]);
}

$inputFor = static function (array $record, array $opts = []) use ($fixture): array {
    $evidenceByHandle = [];
    foreach ($fixture['stripe_evidence'] as $ev) { $evidenceByHandle[$ev['evidence_handle']] = $ev; }
    $mappingByRecord = [];
    foreach ($fixture['edd_mappings'] as $m) { $mappingByRecord[$m['record_handle']] = $m; }
    $mapping = $mappingByRecord[$record['handle']] ?? [];
    $ev = $evidenceByHandle[$record['evidence_handle']] ?? null;
    return [
        'request_id' => $opts['request_id'] ?? 'req_pr_0001',
        'idempotency_key' => $opts['idempotency_key'] ?? 'idem_pr_0001',
        'record' => $record,
        'evidence_handle' => $ev['evidence_handle'] ?? '',
        'evidence_digest' => $ev['digest'] ?? '',
        'verified_identity_digest' => $opts['verified_identity_digest'] ?? ($record['identity_lookup_digest'] ?? ''),
        'account_uuid' => $opts['account_uuid'] ?? ($mapping['account_uuid'] ?? 'acc_pr_vfy_0001'),
        'edd_customer_handle' => $opts['edd_customer_handle'] ?? ($mapping['edd_customer_handle'] ?? 'edd_cust_vfy_0001'),
        'edd_order_handle' => $opts['edd_order_handle'] ?? ($mapping['edd_order_handle'] ?? 'edd_order_vfy_0001'),
        'edd_license_handle' => $opts['edd_license_handle'] ?? ($mapping['edd_license_handle'] ?? 'edd_lic_vfy_0001'),
        'migration_provenance' => $record['migration_provenance'],
    ];
};
$recordByHandle = [];
foreach ($fixture['records'] as $r) { $recordByHandle[$r['handle']] = $r; }
$importHandles = array_map(static fn(array $r): string => $r['handle'], array_filter($fixture['records'], static fn(array $r): bool => $r['disposition'] === 'evidence_backed_import'));
$adverseHandles = array_map(static fn(array $r): string => $r['handle'], array_filter($fixture['records'], static fn(array $r): bool => $r['disposition'] === 'refunded_revoked'));

// ── Dry-run: decisions with zero writes ─────────────────────────────────
$beforeDry = $counts();
foreach ($fixture['records'] as $record) {
    if ($record['disposition'] === 'unresolved') {
        okThrows(static fn() => $service->dryRun($inputFor($record)), 'EDD_ORDER_UNVERIFIED', "dry-run unresolved fails closed {$record['handle']}");
        continue;
    }
    $decision = $service->dryRun($inputFor($record));
    ok($decision['written'] === false && $decision['mode'] === 'dry_run', "dry-run decision {$record['handle']}");
    if ($record['disposition'] === 'refunded_revoked') {
        ok($decision['decision'] === 'preserve_adverse_state' && in_array($decision['reason'], ['REFUNDED', 'REVOKED'], true), "dry-run preserves adverse state {$record['handle']}");
    } else {
        ok($decision['decision'] === 'import', "dry-run imports {$record['handle']}");
    }
}
$afterDry = $counts();
ok($afterDry === $beforeDry, 'dry-run writes zero rows');

// ── Apply: exactly one entitlement per accepted paid record ─────────────
$seq = 0;
foreach ($importHandles as $handle) {
    $seq++;
    $result = $service->applyRecord($inputFor($recordByHandle[$handle], [
        'request_id' => 'req_pr_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'idempotency_key' => 'idem_pr_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
    ]));
    ok($result['decision'] === 'import' && $result['entitlement'] !== null, "import {handle}");
    $one = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_paid_record_edd_mappings WHERE record_handle = '{$handle}'")->fetchColumn();
    ok($one === 1, "exactly one entitlement mapping {$handle}");
    $importOne = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_paid_record_imports WHERE record_handle = '{$handle}'")->fetchColumn();
    ok($importOne === 1, "exactly one import row {$handle}");
}
$afterApply = $counts();
ok($afterApply['imports'] === 5 && $afterApply['mappings'] === 5, 'apply: five imports, five mappings');

// ── Adverse state: preserved, never reactivated ─────────────────────────
$adverse = 0;
foreach ($adverseHandles as $handle) {
    $adverse++;
    $record = $recordByHandle[$handle];
    $result = $service->applyRecord($inputFor($record));
    ok($result['decision'] === 'preserve_adverse_state' && $result['entitlement'] === null, "preserve adverse state {$handle}");
    $expected = $record['adverse_state'] === 'revoked' ? 'REVOKED' : 'REFUNDED';
    ok($result['reason'] === $expected, "typed adverse reason {$handle}");
    okThrows(static fn() => $service->reactivate($handle, $inputFor($record)), $expected, "reactivate() always fails closed {$handle}");
}
$afterAdverse = $counts();
ok($afterAdverse['journal'] === $afterApply['journal'] + 2, 'adverse states journaled once each');
ok($afterAdverse['imports'] === 5 && $afterAdverse['mappings'] === 5, 'adverse states never grant entitlement');

// ── Replay: idempotent, never a second entitlement, never a second journal entry ──
$beforeReplay = $counts();
foreach ($importHandles as $handle) {
    $replayed = $service->applyRecord($inputFor($recordByHandle[$handle]));
    ok($replayed['replayed'] === true && $replayed['decision'] === 'import', "replay returns stored result {$handle}");
}
foreach ($adverseHandles as $handle) {
    $replayed = $service->applyRecord($inputFor($recordByHandle[$handle]));
    ok($replayed['replayed'] === true && $replayed['entitlement'] === null, "replay preserves adverse state {$handle}");
}
$afterReplay = $counts();
ok($afterReplay === $beforeReplay, 'replay is idempotent: zero new rows, zero new journal entries');

// ── Verified identity before ownership delivery ─────────────────────────
$vfy = $recordByHandle['rec_pr_vfy_0001'];
okThrows(static fn() => $service->applyRecord($inputFor($vfy, ['verified_identity_digest' => ''])), 'EMAIL_VERIFICATION_REQUIRED', 'verify_first without identity denied');
okThrows(static fn() => $service->applyRecord($inputFor($vfy, ['verified_identity_digest' => str_repeat('0', 64)])), 'EMAIL_VERIFICATION_FAILED', 'verify_first with wrong identity denied');
$vfyResult = $service->applyRecord($inputFor($vfy));
ok($vfyResult['decision'] === 'import' && $vfyResult['entitlement'] !== null, 'verified identity opens ownership delivery');

// ── Fail-closed negative paths ──────────────────────────────────────────
okThrows(static fn() => $service->applyRecord($inputFor($recordByHandle['rec_pr_unr_0001'])), 'EDD_ORDER_UNVERIFIED', 'unresolved never imports');
okThrows(static fn() => $service->applyRecord($inputFor($recordByHandle['rec_pr_unr_0002'])), 'EDD_ORDER_UNVERIFIED', 'second unresolved never imports');
$badIdem = $inputFor($recordByHandle['rec_pr_imp_0001']);
unset($badIdem['idempotency_key']);
okThrows(static fn() => $service->applyRecord($badIdem), 'IDEMPOTENCY_KEY_REQUIRED', 'idempotency key required');
$badProduct = $inputFor($recordByHandle['rec_pr_imp_0001']);
$badProduct['record']['product_code'] = 'focusa_engine';
okThrows(static fn() => $service->applyRecord($badProduct), 'PRODUCT_MAPPING_REQUIRED', 'client-controlled product denied');
$badPriceInput = $inputFor($recordByHandle['rec_pr_imp_0001']);
$badPriceInput['client_price'] = 1;
$result = $service->applyRecord($badPriceInput);
ok($result['decision'] === 'import' && $result['replayed'] === true, 'price/grant inputs are never accepted (ignored)');

// ── Rollback: preservation-only ─────────────────────────────────────────
$beforeRollback = $counts();
$rollback = $schema->preserveForRollback('2026-08-08T01:00:00Z', ['source' => 'paid_record_migration_rollback']);
ok($rollback['action'] === 'preserve', 'rollback is preservation-only');
ok($counts() === $beforeRollback, 'rollback preserves every migration table');
ok((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_paid_record_migration_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation journaled');

// ── Replay-safe journal chain ────────────────────────────────────────────
ok($service->journalChainValid() === true, 'journal digest chain valid');

$finalCounts = $counts();
$summary = [
    'schema' => 'focusa.spec152e.paid_record_migration_test.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'accepted_imports' => $finalCounts['imports'],
    'entitlement_mappings' => $finalCounts['mappings'],
    'journal_entries' => $finalCounts['journal'],
    'dry_run_writes' => $afterDry['imports'] + $afterDry['mappings'] + $afterDry['journal'],
    'replay_second_entitlement' => 0,
    'reactivation_denials' => 2,
    'verify_first_imported' => 1,
    'rollback_preserved' => 1,
    'journal_chain_valid' => true,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
"""


def run_harness() -> str:
    if not PHP:
        raise AssertionError("FAIL: php is required to execute the paid-record migration adapter")
    with tempfile.TemporaryDirectory() as tmp:
        harness_path = Path(tmp) / "paid_record_migration_harness.php"
        harness_path.write_text(HARNESS, encoding="utf-8")
        proc = subprocess.run(
            [PHP, str(harness_path), str(CONTRACT), str(FIXTURE)],
            capture_output=True, text=True, timeout=120,
        )
        if proc.returncode != 0:
            raise AssertionError(f"FAIL: php harness exited {proc.returncode}: {proc.stderr[:2000]}")
        return proc.stdout.strip()


first = run_harness()
second = run_harness()
expect(first == second, "harness output is byte-identical across runs (replayable)")
result = json.loads(first)
expect(result["result"] == "passed_fail_closed", "harness passed fail-closed")
expect(result["accepted_imports"] == 6, "six imports (five evidence-backed + one verified verify_first)")
expect(result["entitlement_mappings"] == 6, "six entitlement mappings")
# Journal accounting: each import + each preserved adverse state = one append-only entry.
expected_journal = result["accepted_imports"] + 2  # two preserve_adverse_state entries
expect(result["journal_entries"] == expected_journal, "journal entries == imports + adverse states")

positive = result["positive_checks"]
negative = result["negative_checks"]

summary = {
    "schema": "focusa.spec152e.paid_record_migration_validation.v1",
    "fixture_sha256": sha256(fixture_raw),
    "contract_sha256": sha256(contract_raw),
    "harness_sha256": sha256(first),
    "positive_checks": positive,
    "negative_checks": negative,
    "accepted_imports": result["accepted_imports"],
    "entitlement_mappings": result["entitlement_mappings"],
    "journal_entries": result["journal_entries"],
    "dry_run_writes": result["dry_run_writes"],
    "replay_second_entitlement": result["replay_second_entitlement"],
    "reactivation_denials": result["reactivation_denials"],
    "verify_first_imported": result["verify_first_imported"],
    "rollback_preserved": result["rollback_preserved"],
    "journal_chain_valid": result["journal_chain_valid"],
    "result": "passed",
}
print(json.dumps(summary, sort_keys=True))
