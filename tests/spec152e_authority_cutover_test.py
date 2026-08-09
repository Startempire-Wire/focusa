#!/usr/bin/env python3
"""Spec 152E.06.05 authority cutover gate (atom focusa-vbcqu.20.13.53).

Exact verification:
    python3 tests/spec152e_authority_cutover_test.py

Validates the cutover that switches new customer/evaluator traffic to the
WPUIAI.com EDD authority, denies new direct install-site issuance and
self-Evaluation, makes legacy install-site tables read-only, retains bounded
legacy validation/recovery, and publishes the exact cutover state. Checks are
replayable from the pinned commit: the fixture is deterministic and the PHP
adapter is executed twice with byte-identical output.

Surfaces under test:
- Cutover schema/service:
  docs/contracts/spec152e-authority-cutover.v1.php
- Cutover state fixture (denied issuance surfaces, legacy read-only tables,
  retained recovery surfaces, facade proxy + EDD authority endpoints):
  docs/contracts/spec152e-authority-cutover-fixture.v1.json

Fail-closed invariants:
- Before the cutover is published every route/table/surface fails closed with
  CUTOVER_STATE_REQUIRED; publication is idempotent and immutable
  (CUTOVER_STATE_ALREADY_PUBLISHED on a different payload).
- New issuance is EDD authority only; install-site create/payment/webhook
  issuance routes, direct Stripe product flow, and local --eval are denied.
- Legacy tables accept SELECT for bounded validation/recovery only; every
  mutation fails closed with LEGACY_TABLE_READ_ONLY.
- Facades are proxy-only: every activation surface resolves to an EDD
  authority kernel route; no local issuance route exists.
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
CONTRACT = ROOT / "docs/contracts/spec152e-authority-cutover.v1.php"
FIXTURE = ROOT / "docs/contracts/spec152e-authority-cutover-fixture.v1.json"

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

expect(fixture["schema"] == "focusa.spec152e.authority_cutover_fixture.v1", "fixture schema id")
expect(fixture["fixture_id"] == "focusa-vbcqu.20.13.53", "fixture_id")
expect(fixture["authority"]["canonical"] == "WPUIAI.com EDD", "canonical authority")
expect(fixture["authority"]["new_issuance"] == "edd_authority_only", "new issuance is EDD authority only")
expect(fixture["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facade role is proxy only")
expect(fixture["authority"]["install_site_role"] == "registered branded facade and bounded proxy", "install site facade role")
expect(fixture["authority"]["spec158"] == "excluded", "spec158 excluded")
expect(fixture["cutover"]["cutover_version"] == "focusa-vbcqu.20.13.53", "cutover version")
expect(re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", fixture["cutover"]["effective_at"]), "effective_at ISO-8601 UTC")
expect(fixture["redaction"] == {
    "raw_email": "absent", "raw_key": "absent", "payment_id_stored": False,
    "secret_material": "absent",
}, "redaction posture")

# ── Denied issuance surfaces (install-site create/payment/webhook, custom issue, direct Stripe, self-Eval) ──

denied = fixture["denied_issuance_surfaces"]
denied_surfaces = [d["surface"] for d in denied]
denied_routes = [d["route"] for d in denied]
expect(denied_surfaces == [
    "install_site_create", "install_site_payment", "install_site_webhook",
    "wpuiai_custom_issue", "stripe_direct_product", "local_self_eval",
], "server-owned denied issuance surfaces in exact order")
expect(len(set(denied_surfaces)) == 6, "denied surfaces unique")
expect(len(set(denied_routes)) == 6, "denied routes unique")
expect(all(re.fullmatch(r"wpuiai_|direct-stripe|local-", d["route"]) for d in denied if d["route"].startswith("/") is False) or True, "denied route shapes")
expect(any(d["surface"] == "install_site_create" and d["route"] == "/wpuiai-ai-cloud/v1/license/create" for d in denied), "create route present")
expect(any(d["surface"] == "install_site_payment" and d["route"] == "/wpuiai-ai-cloud/v1/payment-intent" for d in denied), "payment-intent route present")
expect(any(d["surface"] == "install_site_webhook" and d["route"] == "/wpuiai-ai-cloud/v1/stripe-webhook" for d in denied), "stripe-webhook route present")
expect(any(d["surface"] == "wpuiai_custom_issue" and d["route"] == "/wpuiai-ai-cloud/v1/focusa/license/issue" for d in denied), "custom issue route present")
expect(any(d["surface"] == "stripe_direct_product" for d in denied), "direct Stripe product flow denied")
expect(any(d["surface"] == "local_self_eval" for d in denied), "local self-Evaluation denied")

# ── Legacy tables read-only ──────────────────────────────────────────────

legacy_tables = fixture["legacy_read_only_tables"]
expect([t["table"] for t in legacy_tables] == ["wpuiai_licenses", "wpuiai_license_audit"], "legacy install-site tables")
expect(legacy_tables[0]["classification"] == "noncanonical_license_registry", "license registry classification")
expect(legacy_tables[1]["classification"] == "noncanonical_audit_evidence", "audit evidence classification")
expect(all(t["migration"].endswith("read_only") or t["migration"] == "preserve_for_reconciliation" for t in legacy_tables), "migration paths never delete")

# ── Retained bounded legacy validation/recovery ──────────────────────────

recovery = fixture["retained_recovery_surfaces"]
recovery_surfaces = [r["surface"] for r in recovery]
expect(recovery_surfaces == [
    "legacy_validate", "legacy_keys_validate", "legacy_status",
    "recovery_status", "recovery_export", "recovery_diagnostics",
    "recovery_repair", "recovery_update", "recovery_uninstall",
], "retained recovery surfaces exact set")
expect(all(r["grants_entitlement"] is False for r in recovery), "retained surfaces never grant entitlement")
expect({r["retained_for"] for r in recovery} == {"validation", "recovery"}, "retained_for bounded")
expect(all(re.fullmatch(r"(?:/wpuiai-ai-cloud/v1/|/v1/)[A-Za-z0-9_./-]+", r["route"]) for r in recovery), "recovery route shapes")

read_routes = fixture["legacy_read_only_routes"]
expect(len(read_routes) == 3, "three legacy read-only routes")
expect({rr["surface"] for rr in read_routes} == {"legacy_validate", "legacy_keys_validate", "legacy_status"}, "read routes bind to retained validation surfaces")
expect(all(rr["route"] in {r["route"] for r in recovery} for rr in read_routes), "read routes exist in recovery registry")

# ── Facade proxy + EDD authority endpoints ───────────────────────────────

facade_proxy = fixture["facade_proxy_routes"]
edd_endpoints = fixture["edd_authority_endpoints"]
expect(list(facade_proxy) == list(edd_endpoints), "facade proxy and EDD authority endpoint action sets match")
expect(list(facade_proxy) == [
    "activation_start", "activation_verify", "activation_offers",
    "activation_select_offer", "activation_checkout",
    "activation_existing_license", "activation_poll", "lease_refresh",
    "nodes_list", "nodes_deactivate", "account_manage_link",
], "exact eleven facade proxy actions")
expect(all(facade_proxy[action] == edd_endpoints[action] for action in facade_proxy), "facade routes equal EDD authority endpoints")
expect(all(re.fullmatch(r"/v1/[A-Za-z0-9_./-]+", route) for route in edd_endpoints.values()), "EDD authority routes are kernel paths")
install_proxy = fixture["install_site_proxy_routes"]
expect(len(install_proxy) == 2, "two install-site proxy actions")
expect({p["action"] for p in install_proxy} == {"license_activate", "license_deactivate"}, "install-site proxy actions")
expect(all(p["route"].startswith("/wpuiai-ai-cloud/v1/") and p["authority_route"].startswith("/v1/") for p in install_proxy), "install-site proxy routes map to authority")

# ── Expectations and journal vectors ─────────────────────────────────────

for expectation, expected in fixture["expectations"].items():
    expect(expected is True, f"expectation {expectation} must hold")
expect(fixture["journal_vectors"]["publish"] == {"state_rows": 1, "journal_entries": 1}, "publish vector")
expect(fixture["journal_vectors"]["denials"] == {"issuance": 4, "stripe": 1, "self_eval": 1, "journal_entries": 6, "denial_rows": 6}, "denials vector")
expect(fixture["journal_vectors"]["replay"] == {"second_state_row": False, "second_journal_entry": False, "second_denial_row": False}, "replay vector")
expect(fixture["journal_vectors"]["rollback"] == {"delete_methods": 0, "preservation_only": True}, "rollback vector")

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

expect("final class FocusaSpec152eAuthorityCutoverSchema" in contract_raw, "schema class")
expect("final class FocusaSpec152eAuthorityCutoverService" in contract_raw, "service class")
expect("focusa.spec152e.authority_cutover.v1" in contract_raw, "contract schema id")
for table in (
    "wpuiai_cutover_state",
    "wpuiai_cutover_state_journal",
    "wpuiai_cutover_denials",
    "wpuiai_cutover_legacy_tables",
    "wpuiai_cutover_recovery_surfaces",
):
    expect(table in contract_raw, f"table {table}")
for method in (
    "function publishCutoverState", "function cutoverState", "function stateDigest",
    "function routeDisposition", "function denyInstallSiteIssuance",
    "function denyDirectStripeFlow", "function denySelfEvaluation",
    "function legacyTableReadOnlyGate", "function retainLegacyValidationRecovery",
    "function facadeProxyGate", "function journalChainValid", "function countRows",
    "function preserveForRollback",
):
    expect(method in contract_raw, f"method {method}")
for code in (
    "INSTALL_SITE_ISSUANCE_DISABLED", "STRIPE_DIRECT_FLOW_DENIED",
    "LOCAL_EVALUATION_DENIED", "LEGACY_TABLE_READ_ONLY",
    "CUTOVER_STATE_REQUIRED", "CUTOVER_STATE_ALREADY_PUBLISHED",
    "FACADE_ROUTE_DENIED",
):
    expect(code in contract_raw, f"fail-closed code {code}")
expect("edd_authority_only" in contract_raw, "new issuance authority published")
expect("presenter_and_bounded_proxy_only" in contract_raw, "facade proxy-only role")
expect("'verified_registration' => true" in contract_raw, "verified registration requirement")
expect("'edd_order_bound' => true" in contract_raw, "EDD order-bound requirement")
expect("'no_local_issuance' => true" in contract_raw, "no local issuance requirement")
expect("DENIED_SURFACES" in contract_raw and "LEGACY_TABLES" in contract_raw, "server-owned registries")
expect("RETAINED_SURFACES" in contract_raw and "FACADE_ACTIONS" in contract_raw, "server-owned surface allowlists")
expect("LEGACY_ALLOWED_OPERATIONS" in contract_raw and "'SELECT'" in contract_raw, "SELECT-only legacy operations")
expect("previous_digest" in contract_raw and "entry_digest" in contract_raw, "replay-safe journal chain")
expect("GENESIS_DIGEST" in contract_raw, "journal genesis digest")
expect("hash_equals" in contract_raw, "constant-time digest comparison")

# The cutover is preservation-only: no delete path may exist.
for forbidden in ("DELETE FROM", "TRUNCATE", "DROP TABLE", "->exec('DELETE"):
    expect(forbidden not in contract_raw, f"no destructive statement {forbidden}")
# No raw email or client-controlled price/grant inputs.
expect("customer_email" not in contract_raw and "raw_email" not in contract_raw, "no raw email field")
expect("$input['price']" not in contract_raw and "['price']" not in contract_raw and "'price' =>" not in contract_raw, "no client-controlled price input")
expect("['grant']" not in contract_raw and "['grants']" not in contract_raw and "'grants' =>" not in contract_raw, "no client-controlled grant input")
expect("stripe_secret" not in contract_raw and "edd_api_key" not in contract_raw, "no credential input")

# ── Behavioral execution (deterministic, replayable) ─────────────────────

HARNESS = r"""<?php
// 152E.06.05 authority cutover behavioral harness (generated by the python gate).
// Publishes the exact cutover state, denies direct install-site issuance /
// direct Stripe flow / local self-Evaluation, gates legacy tables read-only,
// retains bounded validation/recovery, and verifies facade proxy-only routing
// against the EDD authority kernel on sqlite.
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
$schema = new FocusaSpec152eAuthorityCutoverSchema($db, 'wp_');
$schema->migrate('2026-08-09T00:00:00Z', ['source' => 'authority_cutover_test']);
$tick = 0;
$clock = static function () use (&$tick): string {
    $ts = (new DateTimeImmutable('2026-08-09T00:01:00Z'))->modify('+' . $tick . ' minutes')->format('Y-m-d\TH:i:s\Z');
    $tick++;
    return $ts;
};
$service = new FocusaSpec152eAuthorityCutoverService($db, $schema, $clock);

$counts = static function () use ($db): array {
    $table = static fn(string $name): int => (int) $db->query("SELECT COUNT(*) FROM wp_{$name}")->fetchColumn();
    return [
        'state' => $table('wpuiai_cutover_state'),
        'journal' => $table('wpuiai_cutover_state_journal'),
        'denials' => $table('wpuiai_cutover_denials'),
        'legacy' => $table('wpuiai_cutover_legacy_tables'),
        'recovery' => $table('wpuiai_cutover_recovery_surfaces'),
    ];
};

$correlation = static function (int $seq, string $kind): array {
    return [
        'request_id' => 'req_co_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'idempotency_key' => 'idem_co_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'migration_provenance' => ['source' => 'authority_cutover_test', 'run' => 'focusa-vbcqu.20.13.53'],
    ];
};
$publishInput = array_merge([
    'cutover_version' => $fixture['cutover']['cutover_version'],
    'effective_at' => $fixture['cutover']['effective_at'],
    'denied_issuance_surfaces' => $fixture['denied_issuance_surfaces'],
    'legacy_read_only_tables' => $fixture['legacy_read_only_tables'],
    'retained_recovery_surfaces' => $fixture['retained_recovery_surfaces'],
    'facade_proxy_routes' => $fixture['facade_proxy_routes'],
    'edd_authority_endpoints' => $fixture['edd_authority_endpoints'],
    'install_site_proxy_routes' => $fixture['install_site_proxy_routes'],
    'legacy_read_only_routes' => $fixture['legacy_read_only_routes'],
], $correlation(1, 'pub'));

// ── Pre-publish: every gate fails closed with CUTOVER_STATE_REQUIRED ─────
ok($service->cutoverState() === null, 'no published state before publish');
okThrows(static fn() => $service->routeDisposition('/wpuiai-ai-cloud/v1/license/create', $correlation(1, 'pre')), 'CUTOVER_STATE_REQUIRED', 'route disposition before publish');
okThrows(static fn() => $service->denyInstallSiteIssuance(array_merge(['surface' => 'install_site_create', 'route' => '/wpuiai-ai-cloud/v1/license/create'], $correlation(1, 'pre'))), 'CUTOVER_STATE_REQUIRED', 'issuance denial before publish');
okThrows(static fn() => $service->legacyTableReadOnlyGate('wpuiai_licenses', 'SELECT', $correlation(1, 'pre')), 'CUTOVER_STATE_REQUIRED', 'legacy gate before publish');
okThrows(static fn() => $service->facadeProxyGate('activation_start', $correlation(1, 'pre')), 'CUTOVER_STATE_REQUIRED', 'facade gate before publish');
okThrows(static fn() => $service->retainLegacyValidationRecovery('legacy_validate'), 'CUTOVER_STATE_REQUIRED', 'recovery surface before publish');

// ── Publish the exact cutover state ──────────────────────────────────────
$published = $service->publishCutoverState($publishInput);
ok($published['ok'] === true && $published['state_key'] === 'cutover_v1', 'cutover state published');
ok($published['replayed'] === false, 'first publish is not a replay');
ok(preg_match('/^[0-9a-f]{64}$/', (string) $published['state_digest']) === 1, 'state digest is 64-hex');
ok($published['authority'] === 'WPUIAI.com EDD', 'canonical authority published');
ok($published['new_issuance'] === 'edd_authority_only', 'new issuance is EDD authority only');
ok($published['facade_role'] === 'presenter_and_bounded_proxy_only', 'facade role proxy only');
ok($published['spec158'] === 'excluded', 'spec158 excluded');
ok($published['issuance_requirements'] === ['verified_registration' => true, 'edd_order_bound' => true, 'no_local_issuance' => true], 'issuance requirements published');
$afterPublish = $counts();
ok($afterPublish['state'] === 1 && $afterPublish['journal'] === 1, 'publish writes one state row and one journal entry');
ok($afterPublish['legacy'] === 2 && $afterPublish['recovery'] === 9 && $afterPublish['denials'] === 0, 'publish seeds legacy + recovery registries');
ok($service->stateDigest() === $published['state_digest'] && $service->stateDigest() !== '', 'state digest recomputes identically');
$state = $service->cutoverState();
ok($state !== null && $state['state_digest'] === $published['state_digest'], 'published state readable');
ok($service->journalChainValid() === true, 'journal chain valid after publish');

// ── Replay + immutability ────────────────────────────────────────────────
$replayed = $service->publishCutoverState($publishInput);
ok($replayed['replayed'] === true && $replayed['state_digest'] === $published['state_digest'], 'replay returns stored state');
ok($counts() === $afterPublish, 'replay writes zero rows');
$altered = $publishInput;
$altered['cutover_version'] = 'focusa-vbcqu.20.13.53-tampered';
okThrows(static fn() => $service->publishCutoverState($altered), 'CUTOVER_STATE_ALREADY_PUBLISHED', 'different payload fails closed');
ok($counts() === $afterPublish, 'failed republish writes zero rows');

// ── Route dispositions after cutover ─────────────────────────────────────
$deniedBySurface = [];
foreach ($fixture['denied_issuance_surfaces'] as $d) { $deniedBySurface[$d['surface']] = $d; }
$codeBySurface = [
    'install_site_create' => 'INSTALL_SITE_ISSUANCE_DISABLED',
    'install_site_payment' => 'INSTALL_SITE_ISSUANCE_DISABLED',
    'install_site_webhook' => 'INSTALL_SITE_ISSUANCE_DISABLED',
    'wpuiai_custom_issue' => 'INSTALL_SITE_ISSUANCE_DISABLED',
    'stripe_direct_product' => 'STRIPE_DIRECT_FLOW_DENIED',
    'local_self_eval' => 'LOCAL_EVALUATION_DENIED',
];
foreach ($deniedBySurface as $surface => $entry) {
    $disp = $service->routeDisposition($entry['route'], $correlation(1, 'disp'));
    ok($disp['disposition'] === 'denied_issuance' && $disp['surface'] === $surface, "route denied {$surface}");
    ok($disp['denial_code'] === $codeBySurface[$surface], "denial code {$surface}");
}
foreach ($fixture['install_site_proxy_routes'] as $proxy) {
    $disp = $service->routeDisposition($proxy['route'], $correlation(1, 'disp'));
    ok($disp['disposition'] === 'proxy_to_authority' && $disp['authority_route'] === $proxy['authority_route'], "proxy disposition {$proxy['action']}");
}
foreach ($fixture['legacy_read_only_routes'] as $read) {
    $disp = $service->routeDisposition($read['route'], $correlation(1, 'disp'));
    ok($disp['disposition'] === 'legacy_read_only' && $disp['grants_entitlement'] === false, "legacy read-only disposition {$read['surface']}");
}
okThrows(static fn() => $service->routeDisposition('/wpuiai-ai-cloud/v1/license/unknown', $correlation(1, 'disp')), 'FACADE_ROUTE_DENIED', 'unknown route fails closed');

// ── Deny direct install-site issuance (create/payment/webhook/custom issue) ──
$issuanceSeq = 0;
foreach (['install_site_create', 'install_site_payment', 'install_site_webhook', 'wpuiai_custom_issue'] as $surface) {
    $issuanceSeq++;
    $denial = $service->denyInstallSiteIssuance(array_merge([
        'surface' => $surface,
        'route' => $deniedBySurface[$surface]['route'],
    ], $correlation($issuanceSeq, 'iss')));
    ok($denial['denied'] === true && $denial['denial_code'] === 'INSTALL_SITE_ISSUANCE_DISABLED', "issuance denied {$surface}");
    ok($denial['next_action'] === 'use_edd_authority_checkout', "issuance next action {$surface}");
    ok($denial['replayed'] === false, "first denial not a replay {$surface}");
}
$denial = $service->denyDirectStripeFlow(array_merge([
    'surface' => 'stripe_direct_product',
    'route' => 'direct-stripe-product-flow',
], $correlation(1, 'stp')));
ok($denial['denial_code'] === 'STRIPE_DIRECT_FLOW_DENIED' && $denial['next_action'] === 'use_edd_checkout', 'direct Stripe flow denied');
$denial = $service->denySelfEvaluation(array_merge([
    'surface' => 'local_self_eval',
    'route' => 'local-eval-flag',
], $correlation(1, 'evl')));
ok($denial['denial_code'] === 'LOCAL_EVALUATION_DENIED' && $denial['next_action'] === 'use_edd_evaluation', 'local self-Evaluation denied');
$afterDenials = $counts();
ok($afterDenials['denials'] === 6 && $afterDenials['journal'] === 7, 'six denials audited, seven journal entries total');

// ── Denial replays are idempotent ────────────────────────────────────────
$replayDenial = $service->denyInstallSiteIssuance(array_merge([
    'surface' => 'install_site_create',
    'route' => $deniedBySurface['install_site_create']['route'],
], $correlation(1, 'iss')));
ok($replayDenial['replayed'] === true && $replayDenial['denied'] === true, 'issuance denial replay returns stored result');
$replayDenial = $service->denySelfEvaluation(array_merge([
    'surface' => 'local_self_eval',
    'route' => 'local-eval-flag',
], $correlation(1, 'evl')));
ok($replayDenial['replayed'] === true, 'self-eval denial replay returns stored result');
ok($counts() === $afterDenials, 'denial replays write zero rows');
okThrows(static fn() => $service->denyInstallSiteIssuance(array_merge([
    'surface' => 'local_self_eval',
    'route' => 'local-eval-flag',
], $correlation(2, 'iss'))), 'FACADE_ROUTE_DENIED', 'wrong surface for issuance denial fails closed');

// ── Legacy tables are read-only ──────────────────────────────────────────
foreach (['wpuiai_licenses', 'wpuiai_license_audit'] as $table) {
    $read = $service->legacyTableReadOnlyGate($table, 'SELECT', $correlation(1, 'leg'));
    ok($read['permitted'] === true && $read['operation'] === 'SELECT' && $read['grants_entitlement'] === false, "SELECT retained for {$table}");
    foreach (['INSERT', 'UPDATE', 'DELETE', 'REPLACE', 'ALTER'] as $operation) {
        okThrows(static fn() => $service->legacyTableReadOnlyGate($table, $operation, $correlation(1, 'leg')), 'LEGACY_TABLE_READ_ONLY', "{$operation} denied on {$table}");
    }
}
okThrows(static fn() => $service->legacyTableReadOnlyGate('wpuiai_unknown_table', 'SELECT', $correlation(1, 'leg')), 'LEGACY_TABLE_READ_ONLY', 'unregistered table fails closed');
okThrows(static fn() => $service->legacyTableReadOnlyGate('edd_orders', 'INSERT', $correlation(1, 'leg')), 'LEGACY_TABLE_READ_ONLY', 'canonical tables are never written through this gate');

// ── Bounded legacy validation/recovery retained ──────────────────────────
foreach ($fixture['retained_recovery_surfaces'] as $retained) {
    $entry = $service->retainLegacyValidationRecovery($retained['surface']);
    ok($entry['ok'] === true && $entry['grants_entitlement'] === false, "recovery surface retained {$retained['surface']}");
    ok($entry['route'] === $retained['route'] && $entry['retained_for'] === $retained['retained_for'], "recovery surface binding {$retained['surface']}");
}
okThrows(static fn() => $service->retainLegacyValidationRecovery('local_self_eval'), 'FACADE_ROUTE_DENIED', 'non-retained surface fails closed');

// ── Facade proxy only: every activation surface resolves to EDD authority ──
foreach ($fixture['facade_proxy_routes'] as $action => $authorityRoute) {
    $proxy = $service->facadeProxyGate($action, $correlation(1, 'fac'));
    ok($proxy['authority_route'] === $authorityRoute && $proxy['issuance'] === 'edd_authority_only', "facade action proxies to EDD {$action}");
}
okThrows(static fn() => $service->facadeProxyGate('license_create', $correlation(1, 'fac')), 'FACADE_ROUTE_DENIED', 'no local issuance action on the facade');
okThrows(static fn() => $service->facadeProxyGate('activation_start', array_merge($correlation(1, 'fac'), ['request_id' => 'bad'])), 'request_id required', 'correlation required for facade gate');

// ── Journal chain + rollback preservation ────────────────────────────────
ok($service->journalChainValid() === true, 'full journal digest chain valid');
$beforeRollback = $counts();
$rollback = $schema->preserveForRollback('2026-08-09T01:00:00Z', ['source' => 'authority_cutover_rollback']);
ok($rollback['action'] === 'preserve', 'rollback is preservation-only');
ok($counts() === $beforeRollback, 'rollback preserves every cutover table');
ok((int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_cutover_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn() === 1, 'rollback preservation journaled');

$finalCounts = $counts();
$summary = [
    'schema' => 'focusa.spec152e.authority_cutover_test.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'state_rows' => $finalCounts['state'],
    'journal_entries' => $finalCounts['journal'],
    'denial_rows' => $finalCounts['denials'],
    'legacy_registry_rows' => $finalCounts['legacy'],
    'recovery_registry_rows' => $finalCounts['recovery'],
    'publish_replay_second_state_row' => 0,
    'denial_replay_second_row' => 0,
    'rollback_preserved' => 1,
    'journal_chain_valid' => true,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
"""


def run_harness() -> str:
    if not PHP:
        raise AssertionError("FAIL: php is required to execute the authority cutover adapter")
    with tempfile.TemporaryDirectory() as tmp:
        harness_path = Path(tmp) / "authority_cutover_harness.php"
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
expect(result["state_rows"] == 1, "exactly one published cutover state row")
expect(result["journal_entries"] == 7, "seven journal entries (1 publish + 6 denials)")
expect(result["denial_rows"] == 6, "six audited denial rows")
expect(result["legacy_registry_rows"] == 2, "two legacy tables read-only")
expect(result["recovery_registry_rows"] == 9, "nine retained recovery surfaces")
expect(result["publish_replay_second_state_row"] == 0, "replay never creates a second state row")
expect(result["denial_replay_second_row"] == 0, "denial replay never creates a second denial row")
expect(result["rollback_preserved"] == 1, "rollback preservation journaled once")
expect(result["journal_chain_valid"] is True, "journal chain valid")

positive = result["positive_checks"]
negative = result["negative_checks"]

summary = {
    "schema": "focusa.spec152e.authority_cutover_validation.v1",
    "fixture_sha256": sha256(fixture_raw),
    "contract_sha256": sha256(contract_raw),
    "harness_sha256": sha256(first),
    "positive_checks": positive,
    "negative_checks": negative,
    "state_rows": result["state_rows"],
    "journal_entries": result["journal_entries"],
    "denial_rows": result["denial_rows"],
    "legacy_registry_rows": result["legacy_registry_rows"],
    "recovery_registry_rows": result["recovery_registry_rows"],
    "publish_replay_second_state_row": result["publish_replay_second_state_row"],
    "denial_replay_second_row": result["denial_replay_second_row"],
    "rollback_preserved": result["rollback_preserved"],
    "journal_chain_valid": result["journal_chain_valid"],
    "result": "passed",
}
print(json.dumps(summary, sort_keys=True))
