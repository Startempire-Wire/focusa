#!/usr/bin/env python3
"""Spec 152E.06.06 migration canary gate (atom focusa-vbcqu.20.13.54).

Exact verification:
    python3 tests/spec152e_migration_canary_test.py

Validates the bounded migration canary: dry run with zero writes, per-entry
before/after COUNTS/DIGESTS/STATUS comparison against pinned vectors, injected
failure with idempotent retry that can never un-quarantine, unresolved records
that remain quarantined, EDD-vs-authority reconciliation (missing callbacks
cannot leave stale access), rollback-safety proof (rollback cannot undo
verified identity, EDD refund/revoke truth, monotonic sequence, or audit
truth), and the digest-chained migration journal. Checks are replayable from
the pinned commit: fixtures are deterministic and the PHP adapter is executed
twice with byte-identical output.

Surfaces under test:
- Canary cohort, migration journal, reconciler, cutover gates, rollback
  procedure, EDD/refund/sequence truth:
  docs/contracts/spec152e-migration-canary.v1.php
- Canary cohort + EDD/authority truth fixture:
  docs/contracts/spec152e-migration-canary-fixture.v1.json
- Cutover gate dependency (published state consumed as the canary gate):
  docs/contracts/spec152e-authority-cutover.v1.php
  docs/contracts/spec152e-authority-cutover-fixture.v1.json

Fail-closed invariants:
- Before the authority cutover is published every canary/reconcile/rollback
  operation fails closed with CUTOVER_STATE_REQUIRED; the canary binds to the
  published state digest and its exact issuance requirements.
- Dry run writes zero rows; canary applies write exactly one sequence-ledger
  row per applied entry and compare before/after counts/digests/status.
- Injected failure and unresolved records are quarantined with zero writes;
  retry is idempotent and cannot un-quarantine or grant a quarantined record.
- Refund/revoke increment the monotonic sequence and flip status to
  recovery_only; reactivation always fails closed with REFUNDED/REVOKED.
- Reconciliation fails closed with RECONCILIATION_MISMATCH on stale active
  leases for refunded/revoked EDD records or any quarantined record holding a
  lease.
- Rollback proof verifies verified identity, EDD refund/revoke truth, sequence,
  and audit truth are all preserved; rollback is preservation-only.
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
CONTRACT = ROOT / "docs/contracts/spec152e-migration-canary.v1.php"
FIXTURE = ROOT / "docs/contracts/spec152e-migration-canary-fixture.v1.json"
CUTOVER_CONTRACT = ROOT / "docs/contracts/spec152e-authority-cutover.v1.php"
CUTOVER_FIXTURE = ROOT / "docs/contracts/spec152e-authority-cutover-fixture.v1.json"

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


def vector_digest(vector: dict) -> str:
    normalized = {
        "counts": vector["counts"],
        "sequence": int(vector["sequence"]),
        "status": str(vector["status"]),
    }
    return sha256(json.dumps(normalized, sort_keys=True, separators=(",", ":")))


def truth_digest(kind: str, mapping: dict) -> str:
    lines = sorted(f"{h}:{v}" for h, v in mapping.items())
    return sha256(kind + "\n" + "\n".join(lines))


# ── Load artifacts ───────────────────────────────────────────────────────

contract_raw = CONTRACT.read_text(encoding="utf-8")
fixture_raw = FIXTURE.read_text(encoding="utf-8")
cutover_raw = CUTOVER_CONTRACT.read_text(encoding="utf-8")
fixture = json.loads(fixture_raw)
cutover_fixture = json.loads(CUTOVER_FIXTURE.read_text(encoding="utf-8"))

# ── Fixture structure ────────────────────────────────────────────────────

expect(fixture["schema"] == "focusa.spec152e.migration_canary_fixture.v1", "fixture schema id")
expect(fixture["fixture_id"] == "focusa-vbcqu.20.13.54", "fixture_id")
expect(fixture["authority"]["canonical"] == "WPUIAI.com EDD", "canonical authority")
expect(fixture["authority"]["new_issuance"] == "edd_authority_only", "new issuance edd only")
expect(fixture["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facade role proxy only")
expect(fixture["authority"]["spec158"] == "excluded", "spec158 excluded")
expect(fixture["redaction"] == {
    "raw_email": "absent", "raw_key": "absent", "payment_id_stored": False,
    "secret_material": "absent",
}, "redaction posture")
expect(fixture["canary"]["policy"] == "dry_run_then_bounded_canary", "canary policy")
expect(fixture["canary"]["cohort_bound"] == 8, "cohort bound")

cohort = fixture["cohort"]
handles = [e["handle"] for e in cohort]
expect(len(handles) == len(set(handles)) == 8, "eight unique cohort entries")
expect(all(re.fullmatch(r"rec_[a-z0-9_]{4,64}", h) for h in handles), "cohort handle shape")
expect({e["surface"] for e in cohort} <= {"edd_license", "edd_order_item", "authority_account"}, "surfaces bounded")
expect({e["disposition"] for e in cohort} <= {
    "evidence_backed_import", "refunded_revoked", "verify_first", "unresolved",
}, "dispositions bounded")
expect({e["product_code"] for e in cohort} <= {
    "focusa_operator", "uiai_engine_operator", "focusa_uiai_bundle", "focusa_evaluation",
}, "products allowlisted")
expect({e["record_status"] for e in cohort} <= {"active", "refunded", "revoked", "unresolved"}, "record statuses bounded")
by_handle = {e["handle"]: e for e in cohort}

# Every before/after vector digest must recompute identically (pinned, deterministic).
for entry in cohort:
    for side in ("before", "after"):
        vector = {k: entry[side][k] for k in ("counts", "sequence", "status")}
        expect(vector_digest(vector) == entry[side]["digest"], f"pinned {side} digest recomputes {entry['handle']}")
        expect(re.fullmatch(r"[0-9a-f]{64}", entry[side]["digest"]), f"{side} digest 64-hex {entry['handle']}")
    if entry["verified_identity_required"]:
        expect(re.fullmatch(r"[0-9a-f]{64}", entry["identity_digest"]), f"identity digest 64-hex {entry['handle']}")
    else:
        expect(entry["identity_digest"] == "", f"no identity digest when not required {entry['handle']}")

imports = [e for e in cohort if e["disposition"] == "evidence_backed_import" and not e["inject_failure"]]
verify_first = [e for e in cohort if e["disposition"] == "verify_first"]
refunded_revoked = [e for e in cohort if e["disposition"] == "refunded_revoked"]
unresolved = [e for e in cohort if e["disposition"] == "unresolved"]
injected = [e for e in cohort if e["inject_failure"]]
expect(len(imports) == 3 and len(verify_first) == 1, "three imports + one verify_first")
expect(len(refunded_revoked) == 2, "two refunded/revoked cohort entries")
expect(len(unresolved) == 1 and unresolved[0]["handle"] == "rec_cn_unr_0001", "one unresolved cohort entry")
expect(len(injected) == 1 and injected[0]["handle"] == "rec_cn_fail_0001", "one injected-failure cohort entry")

# Refund/revoke transitions must end recovery_only with a sequence increment.
for entry in refunded_revoked:
    expect(entry["after"]["status"] == "recovery_only", f"adverse entry ends recovery_only {entry['handle']}")
    expect(entry["after"]["sequence"] == entry["before"]["sequence"] + 1, f"refund/revoke increments sequence {entry['handle']}")
    expect(entry["after"]["counts"]["sequence_ledger"] == entry["before"]["counts"]["sequence_ledger"] + 1, f"refund/revoke writes one ledger row {entry['handle']}")
# Quarantined entries must end with zero writes.
for entry in unresolved + injected:
    expect(entry["after"] == entry["before"], f"quarantined entry writes nothing {entry['handle']}")

# ── EDD truth vs authority leases ────────────────────────────────────────

edd_map = {row["record_handle"]: row["adverse_state"] for row in fixture["edd_truth"]}
auth_map = {row["record_handle"]: row["status"] for row in fixture["authority_leases"]}
expect(set(edd_map) == {e["handle"] for e in refunded_revoked}, "edd truth covers refunded/revoked cohort")
expect({e["handle"] for e in unresolved + injected} & set(auth_map) == set(), "quarantined records hold no lease")
for handle, adverse in edd_map.items():
    expect(auth_map[handle] == "recovery_only", f"edd adverse maps to recovery_only {handle}")
expect(fixture["reconciliation"]["edd_digest"] == truth_digest("edd", edd_map), "pinned edd digest recomputes")
expect(fixture["reconciliation"]["authority_digest"] == truth_digest("authority", auth_map), "pinned authority digest recomputes")
expect(fixture["reconciliation"]["quarantined_count"] == 2, "two quarantined records")

# ── Expectations and journal vectors ─────────────────────────────────────

for expectation in (
    "dry_run_zero_writes", "bounded_canary_converges_zero_loss",
    "before_after_counts_digests_status_compared", "injected_failure_quarantined",
    "retry_idempotent_and_cannot_unquarantine", "unresolved_remain_quarantined",
    "cutover_gate_required", "refund_sequence_increments_and_recovery_only",
    "rollback_cannot_undo_verified_identity", "rollback_cannot_undo_edd_refund_revoke",
    "rollback_cannot_undo_sequence", "rollback_cannot_undo_audit_truth",
):
    expect(fixture["expectations"][expectation] is True, f"expectation {expectation}")
expect(fixture["journal_vectors"]["cohort_size"] == 8, "journal vector cohort size")
expect(fixture["journal_vectors"]["applied_entries"] == 6, "journal vector applied")
expect(fixture["journal_vectors"]["quarantined_entries"] == 2, "journal vector quarantined")
expect(fixture["journal_vectors"]["ledger_rows"] == 6, "journal vector ledger rows")

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

expect("final class FocusaSpec152eMigrationCanarySchema" in contract_raw, "schema class")
expect("final class FocusaSpec152eMigrationCanaryService" in contract_raw, "service class")
expect("focusa.spec152e.migration_canary.v1" in contract_raw, "contract schema id")
for table in (
    "wpuiai_canary_runs",
    "wpuiai_canary_cohort",
    "wpuiai_canary_journal",
    "wpuiai_canary_reconciliation",
    "wpuiai_canary_sequence_ledger",
    "wpuiai_canary_rollback_proof",
):
    expect(table in contract_raw, f"table {table}")
for method in (
    "function startCanary", "function dryRunCanary", "function runCanaryEntry",
    "function reconcile", "function proveRollback", "function reactivate",
    "function canarySummary", "function vectorDigest", "function journalChainValid",
    "function countRows", "function preserveForRollback",
):
    expect(method in contract_raw, f"method {method}")
for code in (
    "CUTOVER_STATE_REQUIRED", "CANARY_BEFORE_MISMATCH", "CANARY_AFTER_MISMATCH",
    "EMAIL_VERIFICATION_REQUIRED", "EMAIL_VERIFICATION_FAILED",
    "RECONCILIATION_MISMATCH", "UNRESOLVED_QUARANTINED",
    "INJECTED_FAILURE_QUARANTINED", "REFUNDED", "REVOKED",
    "REQUEST_ID_REQUIRED", "IDEMPOTENCY_KEY_REQUIRED", "COHORT_BOUND_EXCEEDED",
    "RUN_ALREADY_STARTED", "CANARY_RUN_REQUIRED", "ROLLBACK_SAFETY_PROOF_FAILED",
):
    expect(code in contract_raw, f"fail-closed code {code}")
expect("previous_digest" in contract_raw and "entry_digest" in contract_raw, "replay-safe journal chain")
expect("GENESIS_DIGEST" in contract_raw, "journal genesis digest")
expect("hash_equals" in contract_raw, "constant-time digest comparison")
expect("wpuiai_cutover_state" in contract_raw, "canary respects the published cutover gate table")
expect("new_issuance" in contract_raw and "edd_authority_only" in contract_raw, "cutover issuance requirement asserted")
expect("facade_role" in contract_raw and "presenter_and_bounded_proxy_only" in contract_raw, "cutover facade role asserted")
expect("spec158" in contract_raw and "excluded" in contract_raw, "spec158 excluded asserted")

# reactivate() can only fail closed: its body contains no write statements.
reactivate_start = contract_raw.index("function reactivate")
reactivate_end = contract_raw.index("function canarySummary")
reactivate_body = contract_raw[reactivate_start:reactivate_end]
expect("INSERT INTO" not in reactivate_body and "UPDATE" not in reactivate_body and "exec(" not in reactivate_body, "reactivate() can only fail closed")

# Preservation-only: no destructive path may exist anywhere in the contract.
for forbidden in ("DELETE FROM", "TRUNCATE", "DROP TABLE", "->exec('DELETE"):
    expect(forbidden not in contract_raw, f"no destructive statement {forbidden}")
# No raw email or client-controlled price/grant inputs.
expect("customer_email" not in contract_raw and "raw_email" not in contract_raw, "no raw email field")
expect("['price']" not in contract_raw and "'price' =>" not in contract_raw, "no client-controlled price input")
expect("['grant']" not in contract_raw and "['grants']" not in contract_raw and "'grant' =>" not in contract_raw, "no client-controlled grant input")
expect("product_code" in contract_raw and "PRODUCTS" in contract_raw, "server-owned product allowlist")

# ── Behavioral execution (deterministic, replayable) ─────────────────────

HARNESS = r"""<?php
// 152E.06.06 migration canary behavioral harness (generated by the python gate).
// Publishes the authority cutover state (atom 20.13.53 contract), then runs the
// bounded canary: dry run, per-entry before/after comparison, injected failure
// + idempotent retry, unresolved quarantine, reconciliation, rollback-safety
// proof, and journal-chain validation on sqlite.
declare(strict_types=1);
$canaryContract = $argv[1];
$cutoverContract = $argv[2];
$canaryFixturePath = $argv[3];
$cutoverFixturePath = $argv[4];
require_once $canaryContract;
require_once $cutoverContract;
$fixture = json_decode((string) file_get_contents($canaryFixturePath), true, 512, JSON_THROW_ON_ERROR);
$cutoverFixture = json_decode((string) file_get_contents($cutoverFixturePath), true, 512, JSON_THROW_ON_ERROR);
$positive = 0;
$negative = 0;
function ok(bool $condition, string $message): void { global $positive; $positive++; if (!$condition) { fwrite(STDERR, "FAIL: {$message}\n"); exit(1); } }
function okThrows(callable $operation, string $code, string $message): void { global $negative; $negative++; try { $operation(); } catch (Throwable $e) { if ($e->getMessage() === $code) { return; } fwrite(STDERR, "FAIL: {$message} (got {$e->getMessage()})\n"); exit(1); } fwrite(STDERR, "FAIL: {$message} (no throw)\n"); exit(1); }

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$cutoverSchema = new FocusaSpec152eAuthorityCutoverSchema($db, 'wp_');
$cutoverSchema->migrate('2026-08-09T00:00:00Z', ['source' => 'authority_cutover_test']);
$schema = new FocusaSpec152eMigrationCanarySchema($db, 'wp_');
$schema->migrate('2026-08-09T00:00:00Z', ['source' => 'migration_canary_test']);
$tick = 0;
$clock = static function () use (&$tick): string {
    $ts = (new DateTimeImmutable('2026-08-09T01:00:00Z'))->modify('+' . $tick . ' minutes')->format('Y-m-d\TH:i:s\Z');
    $tick++;
    return $ts;
};
$cutoverService = new FocusaSpec152eAuthorityCutoverService($db, $cutoverSchema, $clock);
$service = new FocusaSpec152eMigrationCanaryService($db, $schema, $clock);

$counts = static function () use ($db): array {
    $table = static fn(string $name): int => (int) $db->query("SELECT COUNT(*) FROM wp_{$name}")->fetchColumn();
    return [
        'runs' => $table('wpuiai_canary_runs'),
        'cohort' => $table('wpuiai_canary_cohort'),
        'journal' => $table('wpuiai_canary_journal'),
        'reconciliation' => $table('wpuiai_canary_reconciliation'),
        'ledger' => $table('wpuiai_canary_sequence_ledger'),
        'rollback' => $table('wpuiai_canary_rollback_proof'),
    ];
};
$correlation = static function (int $seq, string $kind): array {
    return [
        'request_id' => 'req_cn_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'idempotency_key' => 'idem_cn_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'migration_provenance' => ['source' => 'migration_canary_test', 'run' => 'focusa-vbcqu.20.13.54'],
    ];
};
$runHandle = $fixture['canary']['run_handle'];
$startInput = array_merge([
    'run_handle' => $runHandle,
    'policy' => $fixture['canary']['policy'],
    'cohort' => $fixture['cohort'],
], $correlation(1, 'start'));

// ── Cutover gate: nothing canary-shaped runs before the authority cutover ──
okThrows(static fn() => $service->startCanary($startInput), 'CUTOVER_STATE_REQUIRED', 'startCanary before cutover publish');
okThrows(static fn() => $service->dryRunCanary(array_merge(['run_handle' => $runHandle], $correlation(1, 'pre'))), 'CUTOVER_STATE_REQUIRED', 'dryRunCanary before cutover publish');
okThrows(static fn() => $service->reconcile(array_merge(['run_handle' => $runHandle, 'recon_handle' => 'recon_cn_0001', 'edd_truth' => [], 'authority_leases' => [], 'quarantined_handles' => []], $correlation(1, 'pre'))), 'CUTOVER_STATE_REQUIRED', 'reconcile before cutover publish');
okThrows(static fn() => $service->proveRollback(array_merge(['run_handle' => $runHandle, 'proof_handle' => 'proof_cn_0001'], $correlation(1, 'pre'))), 'CUTOVER_STATE_REQUIRED', 'proveRollback before cutover publish');
ok($counts() === ['runs' => 0, 'cohort' => 0, 'journal' => 0, 'reconciliation' => 0, 'ledger' => 0, 'rollback' => 0], 'pre-publish gates write zero rows');

// ── Publish the authority cutover state (atom 20.13.53 contract) ─────────
$publishInput = array_merge([
    'cutover_version' => $cutoverFixture['cutover']['cutover_version'],
    'effective_at' => $cutoverFixture['cutover']['effective_at'],
    'denied_issuance_surfaces' => $cutoverFixture['denied_issuance_surfaces'],
    'legacy_read_only_tables' => $cutoverFixture['legacy_read_only_tables'],
    'retained_recovery_surfaces' => $cutoverFixture['retained_recovery_surfaces'],
    'facade_proxy_routes' => $cutoverFixture['facade_proxy_routes'],
    'edd_authority_endpoints' => $cutoverFixture['edd_authority_endpoints'],
    'install_site_proxy_routes' => $cutoverFixture['install_site_proxy_routes'],
    'legacy_read_only_routes' => $cutoverFixture['legacy_read_only_routes'],
], $correlation(1, 'pub'));
$published = $cutoverService->publishCutoverState($publishInput);
ok($published['ok'] === true && $published['state_key'] === 'cutover_v1', 'cutover state published');
ok($published['new_issuance'] === 'edd_authority_only' && $published['facade_role'] === 'presenter_and_bounded_proxy_only' && $published['spec158'] === 'excluded', 'published cutover requirements');

// ── Start the bounded canary ─────────────────────────────────────────────
$started = $service->startCanary($startInput);
ok($started['ok'] === true && $started['replayed'] === false, 'canary started');
ok($started['run_handle'] === $runHandle && $started['policy'] === 'dry_run_then_bounded_canary', 'run envelope');
ok($started['cutover_digest'] === $published['state_digest'], 'run bound to the published cutover digest');
$afterStart = $counts();
ok($afterStart['runs'] === 1 && $afterStart['cohort'] === 8, 'one run, eight bounded cohort entries');
ok($afterStart['journal'] === 1, 'canary_started journaled once');

// start replay: identical cohort returns the stored run, zero rows.
$replayedStart = $service->startCanary($startInput);
ok($replayedStart['replayed'] === true, 'start replay returns stored run');
ok($counts() === $afterStart, 'start replay writes zero rows');

// Bound enforcement + immutable run.
$tooBig = $startInput;
$tooBig['cohort'] = array_merge($startInput['cohort'], array_slice($startInput['cohort'], 0, 2));
okThrows(static fn() => $service->startCanary($tooBig), 'COHORT_BOUND_EXCEEDED', 'cohort bound enforced');
$different = $startInput;
$different['cohort'][0]['handle'] = 'rec_cn_imp_0009';
okThrows(static fn() => $service->startCanary($different), 'RUN_ALREADY_STARTED', 'different cohort fails closed');
ok($counts() === $afterStart, 'failed starts write zero rows');

// ── Dry run: whole-cohort decisions with zero writes ─────────────────────
$dry = $service->dryRunCanary(array_merge(['run_handle' => $runHandle], $correlation(1, 'dry')));
ok($dry['written'] === false, 'dry run writes zero rows');
ok(count($dry['decisions']) === 8, 'dry run covers the whole cohort');
$decisionByHandle = [];
foreach ($dry['decisions'] as $d) { $decisionByHandle[$d['record_handle']] = $d; }
foreach (['rec_cn_imp_0001', 'rec_cn_imp_0002', 'rec_cn_imp_0003'] as $handle) {
    ok($decisionByHandle[$handle]['decision'] === 'import', "dry run imports {$handle}");
}
ok($decisionByHandle['rec_cn_vfy_0001']['decision'] === 'import' && $decisionByHandle['rec_cn_vfy_0001']['identity_gate_required'] === true, 'dry run previews verify_first identity gate');
ok($decisionByHandle['rec_cn_ref_0001']['decision'] === 'preserve_adverse_state' && $decisionByHandle['rec_cn_ref_0001']['reason'] === 'REFUNDED', 'dry run preserves refunded');
ok($decisionByHandle['rec_cn_rev_0001']['decision'] === 'preserve_adverse_state' && $decisionByHandle['rec_cn_rev_0001']['reason'] === 'REVOKED', 'dry run preserves revoked');
ok($decisionByHandle['rec_cn_unr_0001']['decision'] === 'quarantine' && $decisionByHandle['rec_cn_unr_0001']['reason'] === 'UNRESOLVED_QUARANTINED', 'dry run quarantines unresolved');
ok($decisionByHandle['rec_cn_fail_0001']['decision'] === 'quarantine' && $decisionByHandle['rec_cn_fail_0001']['reason'] === 'INJECTED_FAILURE_QUARANTINED', 'dry run quarantines injected failure');
ok($counts() === $afterStart, 'dry run writes zero rows');

// ── Bounded canary apply: before/after counts/digests/status comparison ──
$entryInput = static function (string $handle, int $seq, array $opts = []) use ($runHandle, $correlation): array {
    return array_merge($correlation($seq, 'run'), [
        'run_handle' => $runHandle,
        'entry_handle' => $handle,
        'inject_failure' => $opts['inject_failure'] ?? false,
        'verified_identity_digest' => $opts['verified_identity_digest'] ?? '',
    ]);
};
$entryByHandle = [];
foreach ($fixture['cohort'] as $e) { $entryByHandle[$e['handle']] = $e; }

$seq = 0;
foreach (['rec_cn_imp_0001', 'rec_cn_imp_0002', 'rec_cn_imp_0003'] as $handle) {
    $seq++;
    $expected = $entryByHandle[$handle];
    $result = $service->runCanaryEntry($entryInput($handle, $seq));
    ok($result['decision'] === 'import' && $result['reason'] === null, "import applied {$handle}");
    ok($result['status'] === 'active' && $result['sequence'] === 1, "after status/sequence {$handle}");
    ok($result['compared'] === true, "before/after compared {$handle}");
    ok($result['before_digest'] === $expected['before']['digest'], "before digest matches pinned {$handle}");
    ok($result['after_digest'] === $expected['after']['digest'], "after digest matches pinned {$handle}");
    ok($result['after_vector']['counts']['sequence_ledger'] === 1 && $result['after_vector']['status'] === 'active', "after counts/status vector {$handle}");
    $ledgerRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_sequence_ledger WHERE record_handle = '{$handle}'")->fetchColumn();
    ok($ledgerRows === 1, "exactly one ledger row {$handle}");
}

// verify_first: identity gate before any entitlement.
$vfyHandle = 'rec_cn_vfy_0001';
okThrows(static fn() => $service->runCanaryEntry($entryInput($vfyHandle, 4)), 'EMAIL_VERIFICATION_REQUIRED', 'verify_first without identity denied');
okThrows(static fn() => $service->runCanaryEntry($entryInput($vfyHandle, 4, ['verified_identity_digest' => str_repeat('0', 64)])), 'EMAIL_VERIFICATION_FAILED', 'verify_first with wrong identity denied');
$vfy = $service->runCanaryEntry($entryInput($vfyHandle, 4, ['verified_identity_digest' => $entryByHandle[$vfyHandle]['identity_digest']]));
ok($vfy['decision'] === 'import' && $vfy['status'] === 'active' && $vfy['sequence'] === 1, 'verified identity opens canary apply');

// Refund/revoke: EDD adverse state → sequence increment → recovery_only.
$ref = $service->runCanaryEntry($entryInput('rec_cn_ref_0001', 5));
ok($ref['decision'] === 'preserve_adverse_state' && $ref['reason'] === 'REFUNDED', 'refund preserved as adverse state');
ok($ref['status'] === 'recovery_only' && $ref['sequence'] === 1, 'refund → sequence increment + recovery_only (refresh denied)');
$rev = $service->runCanaryEntry($entryInput('rec_cn_rev_0001', 6));
ok($rev['decision'] === 'preserve_adverse_state' && $rev['reason'] === 'REVOKED' && $rev['status'] === 'recovery_only', 'revoke → recovery_only');

// Unresolved record: quarantined, zero writes.
$unr = $service->runCanaryEntry($entryInput('rec_cn_unr_0001', 7));
ok($unr['decision'] === 'quarantine' && $unr['reason'] === 'UNRESOLVED_QUARANTINED', 'unresolved record quarantined');
ok($unr['status'] === 'none' && $unr['sequence'] === 0 && $unr['after_vector']['counts']['sequence_ledger'] === 0, 'unresolved writes no ledger row');

// Injected failure: quarantined with zero writes.
$fail = $service->runCanaryEntry($entryInput('rec_cn_fail_0001', 8, ['inject_failure' => true]));
ok($fail['decision'] === 'quarantine' && $fail['reason'] === 'INJECTED_FAILURE_QUARANTINED', 'injected failure quarantined');
$failRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_sequence_ledger WHERE record_handle = 'rec_cn_fail_0001'")->fetchColumn();
ok($failRows === 0, 'injected failure writes no ledger row');

// Retry: idempotent, returns the stored quarantine, never un-quarantines.
$retry = $service->runCanaryEntry($entryInput('rec_cn_fail_0001', 9));
ok($retry['replayed'] === true && $retry['decision'] === 'quarantine' && $retry['reason'] === 'INJECTED_FAILURE_QUARANTINED', 'retry returns stored quarantine (cannot un-quarantine)');
$retryUnr = $service->runCanaryEntry($entryInput('rec_cn_unr_0001', 10));
ok($retryUnr['replayed'] === true && $retryUnr['reason'] === 'UNRESOLVED_QUARANTINED', 'unresolved stays quarantined on retry');

// Replay of applied entries: stored outcome, zero new rows.
$replayImp = $service->runCanaryEntry($entryInput('rec_cn_imp_0001', 11));
ok($replayImp['replayed'] === true && $replayImp['decision'] === 'import' && $replayImp['status'] === 'active', 'applied entry replay returns stored result');
$afterApply = $counts();
ok($afterApply['ledger'] === 6 && $afterApply['cohort'] === 8, 'six ledger rows, eight cohort entries after apply');
ok($afterApply['journal'] === 1 + 6 + 2, 'journal = started + six applied + two quarantined');

// ── Convergence: zero customer/license loss, zero authority rollback ─────
$summary = $service->canarySummary(['run_handle' => $runHandle]);
ok($summary['cohort_size'] === 8 && $summary['applied'] === 6 && $summary['quarantined'] === 2 && $summary['pending'] === 0, 'canary converged');
ok($summary['converged'] === true, 'canary converged flag');
ok($summary['ledger_rows'] === 6 && $summary['expected_ledger_rows'] === 6 && $summary['zero_loss'] === true, 'zero customer/license loss');
ok($summary['zero_authority_rollback'] === true, 'zero authority rollback');

// ── Reconciliation: EDD truth vs authority truth ─────────────────────────
$reconInput = array_merge([
    'run_handle' => $runHandle,
    'recon_handle' => 'recon_cn_0001',
    'edd_truth' => $fixture['edd_truth'],
    'authority_leases' => $fixture['authority_leases'],
    'quarantined_handles' => ['rec_cn_unr_0001', 'rec_cn_fail_0001'],
], $correlation(1, 'rec'));
$recon = $service->reconcile($reconInput);
ok($recon['matching'] === true, 'reconciliation matches');
ok($recon['quarantined_count'] === 2, 'reconciliation counts two quarantined');
ok($recon['edd_digest'] === $fixture['reconciliation']['edd_digest'], 'edd digest deterministic (pinned)');
ok($recon['authority_digest'] === $fixture['reconciliation']['authority_digest'], 'authority digest deterministic (pinned)');
$afterRecon = $counts();
ok($afterRecon['reconciliation'] === 1, 'one reconciliation row');
ok($afterRecon['journal'] === $afterApply['journal'] + 1, 'reconciliation journaled once');

// Stale access: an authority lease still active for a refunded EDD record fails closed.
$stale = $reconInput;
$stale['recon_handle'] = 'recon_cn_0002';
$stale['authority_leases'][] = ['record_handle' => 'rec_cn_ref_0001', 'status' => 'active'];
okThrows(static fn() => $service->reconcile($stale), 'RECONCILIATION_MISMATCH', 'stale active lease for refunded record fails closed');
// Quarantined record holding a lease fails closed.
$leased = $reconInput;
$leased['recon_handle'] = 'recon_cn_0003';
$leased['authority_leases'][] = ['record_handle' => 'rec_cn_unr_0001', 'status' => 'active'];
okThrows(static fn() => $service->reconcile($leased), 'RECONCILIATION_MISMATCH', 'quarantined record holding a lease fails closed');
// Quarantined set drift fails closed.
$drifted = $reconInput;
$drifted['recon_handle'] = 'recon_cn_0004';
$drifted['quarantined_handles'] = ['rec_cn_unr_0001'];
okThrows(static fn() => $service->reconcile($drifted), 'RECONCILIATION_MISMATCH', 'quarantined set drift fails closed');
ok($counts() === $afterRecon, 'failed reconciliations write zero rows');

// Reconcile replay is idempotent.
$reconReplay = $service->reconcile($reconInput);
ok($reconReplay['replayed'] === true && $reconReplay['matching'] === true, 'reconcile replay returns stored result');
ok($counts() === $afterRecon, 'reconcile replay writes zero rows');

// ── Rollback-safety proof ────────────────────────────────────────────────
$proofInput = array_merge([
    'run_handle' => $runHandle,
    'proof_handle' => 'proof_cn_0001',
], $correlation(1, 'pro'));
$proof = $service->proveRollback($proofInput);
ok($proof['verified_identity_preserved'] === true, 'rollback cannot undo verified identity');
ok($proof['edd_refund_truth_preserved'] === true, 'rollback cannot undo EDD refund/revoke truth');
ok($proof['sequence_preserved'] === true, 'rollback cannot undo monotonic sequence');
ok($proof['audit_preserved'] === true, 'rollback cannot undo audit truth');
ok(preg_match('/^[0-9a-f]{64}$/D', (string) $proof['proof_digest']) === 1, 'proof digest 64-hex');
$afterProof = $counts();
ok($afterProof['rollback'] === 1, 'one rollback proof row');
ok($afterProof['journal'] === $afterRecon['journal'] + 1, 'rollback proof journaled once');

// Reactivation always fails closed.
okThrows(static fn() => $service->reactivate('rec_cn_ref_0001', ['adverse_state' => 'refunded']), 'REFUNDED', 'refunded record never reactivates');
okThrows(static fn() => $service->reactivate('rec_cn_rev_0001', ['adverse_state' => 'revoked']), 'REVOKED', 'revoked record never reactivates');

// ── Replay-safe journal chain + preservation-only rollback ───────────────
ok($service->journalChainValid() === true, 'journal digest chain valid');
$beforeRollback = $counts();
$preserved = $schema->preserveForRollback('2026-08-09T02:00:00Z', ['source' => 'migration_canary_rollback']);
ok($preserved['action'] === 'preserve', 'rollback is preservation-only');
ok($counts() === $beforeRollback, 'rollback preserves every canary table');
$schemaEvent = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn();
ok($schemaEvent === 1, 'rollback preservation journaled');

$finalCounts = $counts();
$appliedRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_cohort WHERE canary_state = 'applied'")->fetchColumn();
$quarantinedRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_cohort WHERE canary_state = 'quarantined'")->fetchColumn();
$summaryOut = [
    'schema' => 'focusa.spec152e.migration_canary_test.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'cohort_size' => $finalCounts['cohort'],
    'applied_entries' => $appliedRows,
    'quarantined_entries' => $quarantinedRows,
    'ledger_rows' => $finalCounts['ledger'],
    'reconciled_rows' => $finalCounts['reconciliation'],
    'rollback_proof_rows' => $finalCounts['rollback'],
    'dry_run_writes' => 0,
    'replay_second_rows' => 0,
    'journal_chain_valid' => true,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summaryOut, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
"""


def run_harness() -> str:
    if not PHP:
        raise AssertionError("FAIL: php is required to execute the migration canary adapter")
    with tempfile.TemporaryDirectory() as tmp:
        harness_path = Path(tmp) / "migration_canary_harness.php"
        harness_path.write_text(HARNESS, encoding="utf-8")
        proc = subprocess.run(
            [PHP, str(harness_path), str(CONTRACT), str(CUTOVER_CONTRACT), str(FIXTURE), str(CUTOVER_FIXTURE)],
            capture_output=True, text=True, timeout=180,
        )
        if proc.returncode != 0:
            raise AssertionError(f"FAIL: php harness exited {proc.returncode}: {proc.stderr[:2000]}")
        return proc.stdout.strip()


first = run_harness()
second = run_harness()
expect(first == second, "harness output is byte-identical across runs (replayable)")
result = json.loads(first)
expect(result["result"] == "passed_fail_closed", "harness passed fail-closed")
expect(result["cohort_size"] == 8, "bounded cohort size")
expect(result["applied_entries"] == 6, "six applied cohort entries")
expect(result["quarantined_entries"] == 2, "two quarantined cohort entries")
expect(result["ledger_rows"] == 6, "six sequence-ledger rows (one per applied entry)")
expect(result["reconciled_rows"] == 1, "one reconciliation row")
expect(result["rollback_proof_rows"] == 1, "one rollback proof row")
expect(result["dry_run_writes"] == 0, "dry run writes zero rows")
expect(result["replay_second_rows"] == 0, "replays write zero rows")
expect(result["journal_chain_valid"] is True, "journal chain valid")

positive = result["positive_checks"]
negative = result["negative_checks"]

summary = {
    "schema": "focusa.spec152e.migration_canary_validation.v1",
    "fixture_sha256": sha256(fixture_raw),
    "contract_sha256": sha256(contract_raw),
    "harness_sha256": sha256(first),
    "positive_checks": positive,
    "negative_checks": negative,
    "cohort_size": result["cohort_size"],
    "applied_entries": result["applied_entries"],
    "quarantined_entries": result["quarantined_entries"],
    "ledger_rows": result["ledger_rows"],
    "reconciled_rows": result["reconciled_rows"],
    "rollback_proof_rows": result["rollback_proof_rows"],
    "dry_run_writes": result["dry_run_writes"],
    "replay_second_rows": result["replay_second_rows"],
    "journal_chain_valid": result["journal_chain_valid"],
    "result": "passed",
}
print(json.dumps(summary, sort_keys=True))
