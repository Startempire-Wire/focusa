#!/usr/bin/env python3
"""Spec 152E.07.08 final migration and cutover canary acceptance (atom
focusa-vbcqu.20.13.62).

Exact verification:
    python3 tests/spec152e_final_cutover_canary_test.py

Repeats the bounded migration canary under the FINAL committed code, compares
counts/status/digests against the pinned canary vectors and the published
cutover state, verifies legacy recovery through the read-only legacy registry
and the retained recovery surfaces, verifies new EDD issuance stays singular
and authority-backed, and executes a rollback rehearsal that cannot roll back
authority truth. Checks are replayable from the pinned commit: every consumed
fixture is deterministic and the PHP adapter is executed twice with
byte-identical output.

Surfaces consumed read-only (no atom code changes, exact-surface scope):
- Bounded canary + EDD/refund/sequence truth:
  docs/contracts/spec152e-migration-canary.v1.php
  docs/contracts/spec152e-migration-canary-fixture.v1.json
- Authority cutover gate (denied issuance, read-only legacy registry, retained
  recovery surfaces, facade proxy + EDD authority endpoints):
  docs/contracts/spec152e-authority-cutover.v1.php
  docs/contracts/spec152e-authority-cutover-fixture.v1.json
- Evidence-backed paid/legacy/synthetic/refunded record fixtures:
  docs/contracts/spec152e-paid-record-migration-fixture.v1.json
  docs/contracts/spec152e-legacy-customer-fixture.v1.json
  docs/contracts/spec152e-migration-inventory.v1.json
- Facades and clients:
  docs/contracts/spec152e-facade-registry.v1.json
  docs/contracts/spec152e-edd-product-registry.v1.json
  docs/contracts/spec152e-recovery-only-surface.v1.json

Fail-closed invariants (Spec 152E §22.3 cutover steps 6-10, §22.4 rollback,
§23 acceptance, §24 completion gate 11-12):
- The canary runs only under the published cutover state and binds to its
  digest; dry run writes zero rows; each applied entry writes exactly one
  sequence-ledger row; before/after counts/digests/status equal the pinned
  vectors; 6 applied + 2 quarantined = 8 converge with zero loss and zero
  authority rollback.
- Legacy install-site tables are read-only (SELECT only; every mutation fails
  closed with LEGACY_TABLE_READ_ONLY); retained validation/recovery surfaces
  grant no entitlement; legacy read-only routes map to bounded recovery.
- New issuance is EDD authority only: install-site create/payment/webhook,
  custom issue, direct Stripe, and local self-Evaluation all fail closed with
  their exact denial codes; every facade action resolves to an EDD authority
  kernel route with no local issuance action; clients cannot supply
  price/grant/product inputs.
- Rollback rehearsal is preservation-only: proveRollback verifies verified
  identity, EDD refund/revoke truth, monotonic sequence, and audit truth are
  preserved; refunded/revoked records remain recovery_only and reactivation
  fails closed; new issuance fails closed during the rehearsal; no
  DELETE/TRUNCATE/DROP path exists anywhere.
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
CANARY_CONTRACT = ROOT / "docs/contracts/spec152e-migration-canary.v1.php"
CANARY_FIXTURE = ROOT / "docs/contracts/spec152e-migration-canary-fixture.v1.json"
CUTOVER_CONTRACT = ROOT / "docs/contracts/spec152e-authority-cutover.v1.php"
CUTOVER_FIXTURE = ROOT / "docs/contracts/spec152e-authority-cutover-fixture.v1.json"
PAID_FIXTURE = ROOT / "docs/contracts/spec152e-paid-record-migration-fixture.v1.json"
LEGACY_FIXTURE = ROOT / "docs/contracts/spec152e-legacy-customer-fixture.v1.json"
INVENTORY_FIXTURE = ROOT / "docs/contracts/spec152e-migration-inventory.v1.json"
FACADE_FIXTURE = ROOT / "docs/contracts/spec152e-facade-registry.v1.json"
PRODUCT_FIXTURE = ROOT / "docs/contracts/spec152e-edd-product-registry.v1.json"
RECOVERY_FIXTURE = ROOT / "docs/contracts/spec152e-recovery-only-surface.v1.json"

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

canary_raw = CANARY_CONTRACT.read_text(encoding="utf-8")
canary_fixture_raw = CANARY_FIXTURE.read_text(encoding="utf-8")
cutover_raw = CUTOVER_CONTRACT.read_text(encoding="utf-8")
cutover_fixture_raw = CUTOVER_FIXTURE.read_text(encoding="utf-8")
paid_raw = PAID_FIXTURE.read_text(encoding="utf-8")
legacy_raw = LEGACY_FIXTURE.read_text(encoding="utf-8")
inventory_raw = INVENTORY_FIXTURE.read_text(encoding="utf-8")
facade_raw = FACADE_FIXTURE.read_text(encoding="utf-8")
product_raw = PRODUCT_FIXTURE.read_text(encoding="utf-8")
recovery_raw = RECOVERY_FIXTURE.read_text(encoding="utf-8")

canary_fixture = json.loads(canary_fixture_raw)
cutover_fixture = json.loads(cutover_fixture_raw)
paid_fixture = json.loads(paid_raw)
legacy_fixture = json.loads(legacy_raw)
inventory_fixture = json.loads(inventory_raw)
facade_fixture = json.loads(facade_raw)
product_fixture = json.loads(product_raw)
recovery_fixture = json.loads(recovery_raw)

# ── Redaction: no secret or unmasked real-email evidence anywhere ────────

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
SYNTHETIC_KEY_RE = re.compile(r"(?i)focusa_live_[0-9]+_[0-9a-f]+")
PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
GITHUB_TOKEN_RE = re.compile(r"ghp_[A-Za-z0-9]{8,}")
LICENSE_SHAPE_RE = re.compile(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}")

for name, raw in (
    ("canary_fixture", canary_fixture_raw),
    ("cutover_fixture", cutover_fixture_raw),
    ("paid_fixture", paid_raw),
    ("legacy_fixture", legacy_raw),
    ("inventory_fixture", inventory_raw),
    ("facade_fixture", facade_raw),
    ("product_fixture", product_raw),
    ("recovery_fixture", recovery_raw),
    ("canary_contract", canary_raw),
    ("cutover_contract", cutover_raw),
):
    expect(not EMAIL_RE.search(raw), f"no raw email in {name}")
    expect(not SECRET_RE.search(raw), f"no secret prefix in {name}")
    expect(not SYNTHETIC_KEY_RE.search(raw), f"no synthetic focusa_live key in {name}")
    expect(not PRIVATE_KEY_RE.search(raw), f"no private key material in {name}")
    expect(not GITHUB_TOKEN_RE.search(raw), f"no GitHub token in {name}")
    expect(not LICENSE_SHAPE_RE.search(raw), f"no raw license-shaped key in {name}")

# ── Cutover fixture: denied issuance, read-only legacy registry, recovery ─

expect(cutover_fixture["schema"] == "focusa.spec152e.authority_cutover_fixture.v1", "cutover fixture schema id")
expect(cutover_fixture["fixture_id"] == "focusa-vbcqu.20.13.53", "cutover fixture_id")
expect(cutover_fixture["authority"]["canonical"] == "WPUIAI.com EDD", "canonical authority")
expect(cutover_fixture["authority"]["new_issuance"] == "edd_authority_only", "new issuance edd only")
expect(cutover_fixture["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facade role proxy only")
expect(cutover_fixture["authority"]["spec158"] == "excluded", "spec158 excluded")

denied = cutover_fixture["denied_issuance_surfaces"]
denied_surfaces = {d["surface"] for d in denied}
expect(denied_surfaces == {
    "install_site_create", "install_site_payment", "install_site_webhook",
    "wpuiai_custom_issue", "stripe_direct_product", "local_self_eval",
}, "six denied issuance surfaces")

legacy_tables = cutover_fixture["legacy_read_only_tables"]
expect({t["table"] for t in legacy_tables} == {"wpuiai_licenses", "wpuiai_license_audit"}, "legacy tables read-only")
expect(all(t["classification"] in {"noncanonical_license_registry", "noncanonical_audit_evidence"} for t in legacy_tables), "legacy classification")

recovery_surfaces = {s["surface"] for s in cutover_fixture["retained_recovery_surfaces"]}
expect(recovery_surfaces == {
    "legacy_validate", "legacy_keys_validate", "legacy_status", "recovery_status",
    "recovery_export", "recovery_diagnostics", "recovery_repair", "recovery_update",
    "recovery_uninstall",
}, "nine retained recovery surfaces")
expect(all(s["grants_entitlement"] is False for s in cutover_fixture["retained_recovery_surfaces"]), "recovery grants no entitlement")

facade_routes = cutover_fixture["facade_proxy_routes"]
edd_routes = cutover_fixture["edd_authority_endpoints"]
expect(set(facade_routes) == set(edd_routes), "facade proxies cover the EDD authority endpoints")
expect(len(facade_routes) == 11, "eleven facade proxy actions")
expect(all(facade_routes[a] == edd_routes[a] for a in facade_routes), "facade route resolves to EDD authority route")

install_proxy = {p["action"] for p in cutover_fixture["install_site_proxy_routes"]}
expect(install_proxy == {"license_activate", "license_deactivate"}, "install-site proxy actions bounded")

legacy_read_routes = cutover_fixture["legacy_read_only_routes"]
expect({r["route"] for r in legacy_read_routes} == {
    "/wpuiai-ai-cloud/v1/license/validate",
    "/wpuiai-ai-cloud/v1/keys/validate",
    "/wpuiai-ai-cloud/v1/license/status",
}, "three legacy read-only routes")
expect(all(r["surface"] in recovery_surfaces for r in legacy_read_routes), "legacy read routes map to retained recovery")

# ── Migration canary fixture: pinned vectors recompute identically ───────

expect(canary_fixture["schema"] == "focusa.spec152e.migration_canary_fixture.v1", "canary fixture schema id")
expect(canary_fixture["fixture_id"] == "focusa-vbcqu.20.13.54", "canary fixture_id")
cohort = canary_fixture["cohort"]
handles = [e["handle"] for e in cohort]
expect(len(handles) == len(set(handles)) == 8, "eight unique cohort entries")
expect({e["surface"] for e in cohort} <= {"edd_license", "edd_order_item", "authority_account"}, "surfaces bounded")
expect({e["disposition"] for e in cohort} <= {
    "evidence_backed_import", "refunded_revoked", "verify_first", "unresolved",
}, "dispositions bounded")

for entry in cohort:
    for side in ("before", "after"):
        vector = {k: entry[side][k] for k in ("counts", "sequence", "status")}
        expect(vector_digest(vector) == entry[side]["digest"], f"pinned {side} digest recomputes {entry['handle']}")
        expect(re.fullmatch(r"[0-9a-f]{64}", entry[side]["digest"]), f"{side} digest 64-hex {entry['handle']}")
    if entry["verified_identity_required"]:
        expect(re.fullmatch(r"[0-9a-f]{64}", entry["identity_digest"]), f"identity digest 64-hex {entry['handle']}")

edd_map = {row["record_handle"]: row["adverse_state"] for row in canary_fixture["edd_truth"]}
auth_map = {row["record_handle"]: row["status"] for row in canary_fixture["authority_leases"]}
expect(canary_fixture["reconciliation"]["edd_digest"] == truth_digest("edd", edd_map), "pinned edd digest recomputes")
expect(canary_fixture["reconciliation"]["authority_digest"] == truth_digest("authority", auth_map), "pinned authority digest recomputes")
expect(canary_fixture["journal_vectors"] == {
    "cohort_size": 8, "applied_entries": 6, "quarantined_entries": 2,
    "ledger_rows": 6, "reconciled_rows": 1, "rollback_proof_rows": 1, "second_run_rows": 0,
}, "canary journal vectors pinned")
expect(all(canary_fixture["expectations"][k] is True for k in canary_fixture["expectations"]), "all canary expectations hold")

# ── Evidence-backed paid / legacy / synthetic / refunded record fixtures ─

expect(paid_fixture["schema"] == "focusa.spec152e.paid_record_migration_fixture.v1", "paid fixture schema id")
expect(paid_fixture["fixture_id"] == "focusa-vbcqu.20.13.50", "paid fixture_id")
paid_records = paid_fixture["records"]
paid_imports = [r for r in paid_records if r["disposition"] == "evidence_backed_import"]
paid_adverse = [r for r in paid_records if r["disposition"] == "refunded_revoked"]
expect(len(paid_records) == 10, "ten paid records in the fixture")
expect(len(paid_imports) == 5, "five evidence-backed imports")
expect(len(paid_adverse) == 2, "two refunded/revoked paid records")
expect(paid_fixture["journal_vectors"]["apply"]["imports"] == 5, "paid apply imports = 5")
expect(paid_fixture["journal_vectors"]["apply"]["mappings"] == 5, "paid apply mappings = 5")
expect(len(paid_fixture["edd_mappings"]) == 5, "five EDD mappings")
mapped = {m["record_handle"] for m in paid_fixture["edd_mappings"]}
expect(mapped == {r["handle"] for r in paid_imports}, "mappings cover exactly the imports")
expect({r["product_code"] for r in paid_records} <= set(paid_fixture["product_allowlist"]), "paid products allowlisted")
expect(paid_fixture["policy"] == "evidence_backed_paid_import_only", "paid migration policy")

expect(legacy_fixture["schema"] == "focusa.spec152e.legacy_customer_fixture.v1", "legacy fixture schema id")
expect(legacy_fixture["fixture_id"] == "focusa-vbcqu.20.13.51", "legacy fixture_id")
legacy_records = legacy_fixture["records"]
legacy_dispositions = [r["disposition"] for r in legacy_records]
expect(len(legacy_records) == 9, "nine legacy records in the fixture")
expect(legacy_dispositions.count("verify_first") == 3, "three verify_first legacy records")
expect(legacy_dispositions.count("refunded_revoked") == 2, "two refunded/revoked legacy records")
expect("duplicate" in legacy_dispositions and "synthetic_quarantine" in legacy_dispositions and "unresolved" in legacy_dispositions, "duplicate/synthetic/unresolved dispositions present")
expect(legacy_fixture["policy"] == "verified_identity_promotion_only", "legacy promotion policy")
expect({r["surface"] for r in legacy_records} <= {"edd_customer", "edd_order", "edd_license", "install_site_license"}, "legacy surfaces bounded")

expect(inventory_fixture["inventory_id"] == "focusa-vbcqu.20.13.49", "inventory id")
expect(len(inventory_fixture["records"]) == 596, "inventory records = 596")
physical = inventory_fixture["reconciliation"]["physical_record_counts"]
expect(sum(physical.values()) == 596, "physical record counts sum to 596")
disp = inventory_fixture["reconciliation"]["disposition_counts"]
expect(sum(disp.values()) == 596, "disposition counts sum to 596")
expect(disp["refunded_revoked"] == 41 and disp["synthetic_quarantine"] == 34, "refunded/synthetic disposition counts")
expect(disp["unresolved"] == 515 and disp["verify_first"] == 6, "unresolved/verify_first disposition counts")
expect(inventory_fixture["reconciliation"]["destructive_reconciliation_forbidden"] is True, "destructive reconciliation forbidden")
expect(inventory_fixture["rollback"] == {"artifact_only": True, "preserve_authority_truth": True, "preserve_migration_journal": True}, "inventory rollback posture")

# ── Facades and clients ──────────────────────────────────────────────────

expect(recovery_fixture["schema"] == "focusa.spec152e.recovery_only_surface.v1", "recovery surface schema id")
expect(recovery_fixture["authority"]["canonical"] == "WPUIAI.com EDD", "recovery authority canonical")
expect(recovery_fixture["authority"]["spec158"] == "excluded", "recovery spec158 excluded")
expect(all(recovery_fixture["invariants"].values()), "recovery invariants hold")
expect(all(s["allowance"] != "entitlement" for s in recovery_fixture["recovery_surfaces"]), "recovery never grants entitlement")

expect(facade_fixture["schema"] == "focusa.spec152e.facade_registry.v1", "facade registry schema id")
expect(facade_fixture["authority"]["entitlement_issuance"] == "forbidden", "facades cannot issue entitlement")
expect(facade_fixture["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facade role proxy only")
expect(facade_fixture["counts"]["facades"] == 6, "six registered facades")
expect(facade_fixture["counts"]["proxy_routes"] == 11, "eleven facade proxy routes")
expect(set(facade_fixture["proxy_routes"]) == set(cutover_fixture["facade_proxy_routes"]), "facade registry matches cutover proxy routes")
for forbidden in facade_fixture["request_contract"]["forbidden"]:
    expect(forbidden in {"products", "features", "grants", "limits", "edd_download_id",
                         "edd_price_id", "price", "sender_id", "sender_email", "callback_url",
                         "redirect_url", "success_url", "cancel_url", "authority", "credential", "secret"}, f"request contract forbids {forbidden}")
expect(facade_fixture["request_contract"]["unknown_callback"] == "FACADE_CALLBACK_DENIED", "unknown callback fails closed")
expect(facade_fixture["request_contract"]["unknown_origin"] == "FACADE_ORIGIN_DENIED", "unknown origin fails closed")
expect(facade_fixture["request_contract"]["unknown_product"] == "FACADE_PRODUCT_DENIED", "unknown product fails closed")

expect(product_fixture["schema"] == "focusa.spec152e.edd_product_registry.v1", "product registry schema id")
expect(product_fixture["owner"] == "WPUIAI/wpuiai", "product registry owner")
expect(product_fixture["product"] == "focusa-authority", "product identity")
expect(product_fixture["counts"]["catalog_entries"] == 14, "fourteen EDD catalog entries")
expect(product_fixture["counts"]["checkout_enabled"] == 0, "zero checkout-enabled catalog entries")
expect(product_fixture["counts"]["protected_offers"] == 3, "three protected offers")
legacy_classes = product_fixture["legacy_record_classes"]
expect(len(legacy_classes) == 10, "ten legacy record classes")
expect(sum(1 for c in legacy_classes if c["disposition"] == "migrate") == 3, "three migrate classes")
expect(sum(1 for c in legacy_classes if c["disposition"] == "quarantine") == 5, "five quarantine classes")
expect(sum(1 for c in legacy_classes if c["disposition"] == "retire") == 2, "two retire classes")
expect(all(c["disposition"] == "retire" for c in legacy_classes if "refunded" in c["id"] or "revoked" in c["id"]), "refunded/revoked classes retire, never reactivate")
expect(product_fixture["authority"]["runtime_grant"] == "authority_issued_signed_lease", "runtime grant is signed authority lease")
expect(product_fixture["authority"]["customer_commerce_human_key_refund_entitlement"] == "WPUIAI.com EDD", "refund entitlement is EDD")
for controlled in product_fixture["authority"]["caller_controls_forbidden"]:
    expect(controlled in {"edd_download_id", "edd_price_id", "price", "tier", "product",
                          "product_code", "products", "license_type", "license_type_ref",
                          "capability_family", "families", "features", "limits", "node_limit",
                          "sale_status", "refund_policy", "upgrade_policy",
                          "commercial_rights", "evaluation_duration"}, f"caller cannot control {controlled}")

# ── Cross-surface product allowlist consistency ─────────────────────────

canary_products = {e["product_code"] for e in cohort}
paid_products = set(paid_fixture["product_allowlist"])
expect(canary_products <= paid_products, "canary products within the paid allowlist")
expect(paid_products == {"focusa_operator", "uiai_engine_operator", "focusa_uiai_bundle", "focusa_evaluation"}, "server-owned product allowlist")

# ── Contract static invariants ───────────────────────────────────────────

expect("final class FocusaSpec152eMigrationCanaryService" in canary_raw, "canary service class")
expect("final class FocusaSpec152eAuthorityCutoverService" in cutover_raw, "cutover service class")
expect("new_issuance" in cutover_raw and "edd_authority_only" in cutover_raw, "cutover asserts EDD-only issuance")
expect("facade_role" in cutover_raw and "presenter_and_bounded_proxy_only" in cutover_raw, "cutover asserts facade proxy role")
expect("spec158" in cutover_raw and "excluded" in cutover_raw, "spec158 excluded asserted")
expect("wpuiai_cutover_state" in canary_raw, "canary couples to the published cutover state")
for forbidden in ("DELETE FROM", "TRUNCATE", "DROP TABLE", "->exec('DELETE"):
    expect(forbidden not in canary_raw and forbidden not in cutover_raw, f"no destructive statement {forbidden}")
expect("customer_email" not in canary_raw and "raw_email" not in canary_raw, "no raw email field in canary")
expect("['price']" not in canary_raw and "'price' =>" not in canary_raw, "no client-controlled price in canary")
expect("['grant']" not in canary_raw and "['grants']" not in canary_raw and "'grant' =>" not in canary_raw, "no client-controlled grant in canary")
expect("product_code" in canary_raw and "PRODUCTS" in canary_raw, "server-owned product allowlist in canary")

# ── Behavioral execution: final cutover canary (deterministic, replayable) ─

HARNESS = r"""<?php
// 152E.07.08 final cutover canary harness (generated by the python gate).
// Repeats the bounded migration canary under the final committed code with the
// published authority cutover state, compares counts/status/digests against
// the pinned vectors, verifies legacy recovery (read-only legacy registry +
// retained recovery surfaces), verifies new EDD issuance stays singular and
// authority-backed (denied surfaces fail closed, facades proxy to the EDD
// kernel), and executes a rollback rehearsal that cannot roll back authority
// truth. Deterministic and replayable on sqlite.
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
$cutoverSchema->migrate('2026-08-09T00:00:00Z', ['source' => 'final_cutover_canary_test']);
$schema = new FocusaSpec152eMigrationCanarySchema($db, 'wp_');
$schema->migrate('2026-08-09T00:00:00Z', ['source' => 'final_cutover_canary_test']);
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
        'denials' => $table('wpuiai_cutover_denials'),
        'legacy_tables' => $table('wpuiai_cutover_legacy_tables'),
        'recovery_surfaces' => $table('wpuiai_cutover_recovery_surfaces'),
    ];
};
$correlation = static function (int $seq, string $kind): array {
    return [
        'request_id' => 'req_fc_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'idempotency_key' => 'idem_fc_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'migration_provenance' => ['source' => 'final_cutover_canary_test', 'run' => 'focusa-vbcqu.20.13.62'],
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
ok($counts()['runs'] === 0 && $counts()['cohort'] === 0, 'pre-publish gates write zero rows');

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
ok($cutoverService->journalChainValid() === true, 'cutover journal chain valid');
ok($counts()['legacy_tables'] === 2 && $counts()['recovery_surfaces'] === 9, 'legacy tables + recovery surfaces registered');

// ── Repeat the bounded canary under final code ───────────────────────────
$started = $service->startCanary($startInput);
ok($started['ok'] === true && $started['replayed'] === false, 'canary started under final code');
ok($started['run_handle'] === $runHandle && $started['policy'] === 'dry_run_then_bounded_canary', 'run envelope');
ok($started['cutover_digest'] === $published['state_digest'], 'run bound to the published cutover digest');
$afterStart = $counts();
ok($afterStart['runs'] === 1 && $afterStart['cohort'] === 8 && $afterStart['journal'] === 1, 'one immutable run, eight bounded entries, started journaled once');

$dry = $service->dryRunCanary(array_merge(['run_handle' => $runHandle], $correlation(1, 'dry')));
ok($dry['written'] === false && count($dry['decisions']) === 8, 'dry run covers the whole cohort with zero writes');
$decisionByHandle = [];
foreach ($dry['decisions'] as $d) { $decisionByHandle[$d['record_handle']] = $d; }
ok($decisionByHandle['rec_cn_vfy_0001']['identity_gate_required'] === true, 'dry run previews verify_first identity gate');
ok($decisionByHandle['rec_cn_ref_0001']['decision'] === 'preserve_adverse_state' && $decisionByHandle['rec_cn_ref_0001']['reason'] === 'REFUNDED', 'dry run preserves refunded');
ok($decisionByHandle['rec_cn_rev_0001']['decision'] === 'preserve_adverse_state' && $decisionByHandle['rec_cn_rev_0001']['reason'] === 'REVOKED', 'dry run preserves revoked');
ok($decisionByHandle['rec_cn_unr_0001']['reason'] === 'UNRESOLVED_QUARANTINED' && $decisionByHandle['rec_cn_fail_0001']['reason'] === 'INJECTED_FAILURE_QUARANTINED', 'dry run quarantines unresolved + injected failure');
ok($counts() === $afterStart, 'dry run writes zero rows');

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

// Apply: before/after counts/digests/status compared against pinned vectors.
$seq = 0;
foreach (['rec_cn_imp_0001', 'rec_cn_imp_0002', 'rec_cn_imp_0003'] as $handle) {
    $seq++;
    $expected = $entryByHandle[$handle];
    $result = $service->runCanaryEntry($entryInput($handle, $seq));
    ok($result['decision'] === 'import' && $result['status'] === 'active' && $result['sequence'] === 1, "import applied {$handle}");
    ok($result['compared'] === true, "before/after compared {$handle}");
    ok($result['before_digest'] === $expected['before']['digest'] && $result['after_digest'] === $expected['after']['digest'], "digests match pinned {$handle}");
    ok($result['after_vector']['counts']['sequence_ledger'] === 1 && $result['after_vector']['status'] === 'active', "after counts/status vector {$handle}");
    $ledgerRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_sequence_ledger WHERE record_handle = '{$handle}'")->fetchColumn();
    ok($ledgerRows === 1, "exactly one ledger row {$handle}");
}

// verify_first: identity gate before any entitlement.
$vfyHandle = 'rec_cn_vfy_0001';
okThrows(static fn() => $service->runCanaryEntry($entryInput($vfyHandle, 4)), 'EMAIL_VERIFICATION_REQUIRED', 'verify_first without identity denied');
okThrows(static fn() => $service->runCanaryEntry($entryInput($vfyHandle, 4, ['verified_identity_digest' => str_repeat('0', 64)])), 'EMAIL_VERIFICATION_FAILED', 'verify_first with wrong identity denied');
$vfy = $service->runCanaryEntry($entryInput($vfyHandle, 4, ['verified_identity_digest' => $entryByHandle[$vfyHandle]['identity_digest']]));
ok($vfy['decision'] === 'import' && $vfy['status'] === 'active' && $vfy['after_digest'] === $entryByHandle[$vfyHandle]['after']['digest'], 'verified identity opens canary apply');

// Refund/revoke: EDD adverse state → sequence increment → recovery_only.
$ref = $service->runCanaryEntry($entryInput('rec_cn_ref_0001', 5));
ok($ref['decision'] === 'preserve_adverse_state' && $ref['reason'] === 'REFUNDED', 'refund preserved as adverse state');
ok($ref['status'] === 'recovery_only' && $ref['sequence'] === 1, 'refund → sequence increment + recovery_only');
ok($ref['after_digest'] === $entryByHandle['rec_cn_ref_0001']['after']['digest'], 'refund after digest matches pinned');
$rev = $service->runCanaryEntry($entryInput('rec_cn_rev_0001', 6));
ok($rev['decision'] === 'preserve_adverse_state' && $rev['reason'] === 'REVOKED' && $rev['status'] === 'recovery_only', 'revoke → recovery_only');
ok($rev['after_digest'] === $entryByHandle['rec_cn_rev_0001']['after']['digest'], 'revoke after digest matches pinned');

// Unresolved + injected failure: quarantined with zero writes.
$unr = $service->runCanaryEntry($entryInput('rec_cn_unr_0001', 7));
ok($unr['decision'] === 'quarantine' && $unr['reason'] === 'UNRESOLVED_QUARANTINED', 'unresolved record quarantined');
ok($unr['status'] === 'none' && $unr['sequence'] === 0 && $unr['after_vector']['counts']['sequence_ledger'] === 0, 'unresolved writes no ledger row');
$fail = $service->runCanaryEntry($entryInput('rec_cn_fail_0001', 8, ['inject_failure' => true]));
ok($fail['decision'] === 'quarantine' && $fail['reason'] === 'INJECTED_FAILURE_QUARANTINED', 'injected failure quarantined');
$failRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_sequence_ledger WHERE record_handle = 'rec_cn_fail_0001'")->fetchColumn();
ok($failRows === 0, 'injected failure writes no ledger row');

// Retry: idempotent, never un-quarantines.
$retry = $service->runCanaryEntry($entryInput('rec_cn_fail_0001', 9));
ok($retry['replayed'] === true && $retry['reason'] === 'INJECTED_FAILURE_QUARANTINED', 'retry returns stored quarantine');
$afterApply = $counts();
ok($afterApply['ledger'] === 6 && $afterApply['cohort'] === 8, 'six ledger rows, eight cohort entries after apply');
ok($afterApply['journal'] === 1 + 6 + 2, 'journal = started + six applied + two quarantined');

// Convergence: zero customer/license loss, zero authority rollback.
$summary = $service->canarySummary(['run_handle' => $runHandle]);
ok($summary['cohort_size'] === 8 && $summary['applied'] === 6 && $summary['quarantined'] === 2 && $summary['pending'] === 0, 'canary converged');
ok($summary['converged'] === true && $summary['ledger_rows'] === 6 && $summary['expected_ledger_rows'] === 6, 'converged with expected ledger');
ok($summary['zero_loss'] === true && $summary['zero_authority_rollback'] === true, 'zero loss + zero authority rollback');
ok($service->journalChainValid() === true, 'canary journal chain valid');

// ── Reconciliation: EDD truth vs authority truth ─────────────────────────
$reconInput = array_merge([
    'run_handle' => $runHandle,
    'recon_handle' => 'recon_fc_0001',
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
ok($afterRecon['reconciliation'] === 1 && $afterRecon['journal'] === $afterApply['journal'] + 1, 'one reconciliation row, journaled once');
$stale = $reconInput;
$stale['recon_handle'] = 'recon_fc_0002';
$stale['authority_leases'][] = ['record_handle' => 'rec_cn_ref_0001', 'status' => 'active'];
okThrows(static fn() => $service->reconcile($stale), 'RECONCILIATION_MISMATCH', 'stale active lease for refunded record fails closed');
$drifted = $reconInput;
$drifted['recon_handle'] = 'recon_fc_0003';
$drifted['quarantined_handles'] = ['rec_cn_unr_0001'];
okThrows(static fn() => $service->reconcile($drifted), 'RECONCILIATION_MISMATCH', 'quarantined set drift fails closed');
ok($counts() === $afterRecon, 'failed reconciliations write zero rows');

// ── Legacy recovery: read-only legacy registry + retained recovery ───────
$legacyRead = $cutoverService->legacyTableReadOnlyGate('wpuiai_licenses', 'SELECT', $correlation(1, 'leg'));
ok($legacyRead['permitted'] === true && $legacyRead['grants_entitlement'] === false, 'legacy registry SELECT retained, no entitlement');
$auditRead = $cutoverService->legacyTableReadOnlyGate('wpuiai_license_audit', 'SELECT', $correlation(2, 'leg'));
ok($auditRead['permitted'] === true, 'legacy audit table SELECT retained');
foreach (['UPDATE', 'INSERT', 'DELETE'] as $op) {
    okThrows(static fn() => $cutoverService->legacyTableReadOnlyGate('wpuiai_licenses', $op, $correlation(3, 'leg')), 'LEGACY_TABLE_READ_ONLY', "legacy registry {$op} denied");
}
okThrows(static fn() => $cutoverService->legacyTableReadOnlyGate('wpuiai_orders', 'SELECT', $correlation(4, 'leg')), 'LEGACY_TABLE_READ_ONLY', 'unregistered table denied');
$recoverySurfaces = $cutoverFixture['retained_recovery_surfaces'];
foreach ($recoverySurfaces as $retained) {
    $gate = $cutoverService->retainLegacyValidationRecovery($retained['surface']);
    ok($gate['ok'] === true && $gate['grants_entitlement'] === false && $gate['route'] === $retained['route'], "recovery surface {$retained['surface']} grants no entitlement");
}
okThrows(static fn() => $cutoverService->retainLegacyValidationRecovery('recovery_grant'), 'FACADE_ROUTE_DENIED', 'unregistered recovery surface denied');
foreach ($cutoverFixture['legacy_read_only_routes'] as $readRoute) {
    $disposition = $cutoverService->routeDisposition($readRoute['route'], $correlation(5, 'leg'));
    ok($disposition['disposition'] === 'legacy_read_only' && $disposition['grants_entitlement'] === false, "legacy read-only route {$readRoute['route']}");
}
foreach ($cutoverFixture['install_site_proxy_routes'] as $proxy) {
    $disposition = $cutoverService->routeDisposition($proxy['route'], $correlation(6, 'leg'));
    ok($disposition['disposition'] === 'proxy_to_authority' && $disposition['authority_route'] === $proxy['authority_route'], "install-site proxy route {$proxy['route']}");
}

// ── New EDD issuance: singular, authority-backed, denied surfaces closed ──
$deniedBySurface = [];
foreach ($cutoverFixture['denied_issuance_surfaces'] as $denied) { $deniedBySurface[$denied['surface']] = $denied; }
$installSiteDenied = ['install_site_create', 'install_site_payment', 'install_site_webhook', 'wpuiai_custom_issue'];
foreach ($deniedBySurface as $surface => $denied) {
    $disposition = $cutoverService->routeDisposition($denied['route'], $correlation(1, 'iss'));
    ok($disposition['disposition'] === 'denied_issuance' && $disposition['surface'] === $surface, "denied issuance surface {$surface}");
    if (in_array($surface, $installSiteDenied, true)) {
        $deniedCall = $cutoverService->denyInstallSiteIssuance(array_merge(['surface' => $surface, 'route' => $denied['route']], $correlation(2, 'iss')));
        ok($deniedCall['denied'] === true && $deniedCall['denial_code'] === $disposition['denial_code'], "denial journaled {$surface}");
    }
}
okThrows(static fn() => $cutoverService->routeDisposition('/wpuiai-ai-cloud/v1/focusa/local/issue', $correlation(3, 'iss')), 'FACADE_ROUTE_DENIED', 'unregistered issuance route denied');
$stripeDenial = $cutoverService->denyDirectStripeFlow(array_merge($deniedBySurface['stripe_direct_product'], $correlation(4, 'iss')));
ok($stripeDenial['denied'] === true && $stripeDenial['denial_code'] === 'STRIPE_DIRECT_FLOW_DENIED', 'direct Stripe flow denied');
$evalDenial = $cutoverService->denySelfEvaluation(array_merge($deniedBySurface['local_self_eval'], $correlation(5, 'iss')));
ok($evalDenial['denied'] === true && $evalDenial['denial_code'] === 'LOCAL_EVALUATION_DENIED', 'local self-Evaluation denied');
ok($counts()['denials'] === count($installSiteDenied) + 2, 'denials journaled once each (idempotent)');
$replayDenial = $cutoverService->denyInstallSiteIssuance(array_merge($deniedBySurface['install_site_create'], $correlation(2, 'iss')));
ok($replayDenial['replayed'] === true, 'denial replay is idempotent');
ok($counts()['denials'] === count($installSiteDenied) + 2, 'denial replay writes zero new rows');

// Facade proxy gate: every facade action resolves to the EDD authority kernel.
foreach ($cutoverFixture['facade_proxy_routes'] as $action => $route) {
    $proxy = $cutoverService->facadeProxyGate($action, $correlation(1, 'fac'));
    ok($proxy['ok'] === true && $proxy['authority_route'] === $route && $proxy['issuance'] === 'edd_authority_only', "facade action {$action} proxies to EDD authority");
}
okThrows(static fn() => $cutoverService->facadeProxyGate('local_issue', $correlation(2, 'fac')), 'FACADE_ROUTE_DENIED', 'no local issuance facade action');
ok($cutoverService->journalChainValid() === true, 'cutover journal chain valid after issuance checks');

// ── Rollback rehearsal without authority rollback ────────────────────────
$proofInput = array_merge(['run_handle' => $runHandle, 'proof_handle' => 'proof_fc_0001'], $correlation(1, 'pro'));
$proof = $service->proveRollback($proofInput);
ok($proof['verified_identity_preserved'] === true, 'rollback cannot undo verified identity');
ok($proof['edd_refund_truth_preserved'] === true, 'rollback cannot undo EDD refund/revoke truth');
ok($proof['sequence_preserved'] === true, 'rollback cannot undo monotonic sequence');
ok($proof['audit_preserved'] === true, 'rollback cannot undo audit truth');
ok(preg_match('/^[0-9a-f]{64}$/D', (string) $proof['proof_digest']) === 1, 'proof digest 64-hex');
$afterProof = $counts();
ok($afterProof['rollback'] === 1 && $afterProof['journal'] === $afterRecon['journal'] + 1, 'one rollback proof row, journaled once');

// Authority truth stays recovery_only during rehearsal; reactivation fails closed.
$cohortState = static function (string $handle) use ($db): array {
    $stmt = $db->prepare("SELECT canary_state, sequence, outcome_payload FROM wp_wpuiai_canary_cohort WHERE record_handle = :h");
    $stmt->execute([':h' => $handle]);
    $row = $stmt->fetch(PDO::FETCH_ASSOC);
    if ($row === false) { return []; }
    $outcome = json_decode((string) $row['outcome_payload'], true);
    return ['state' => $row['canary_state'], 'status' => $outcome['status'] ?? 'none', 'sequence' => (int) $row['sequence']];
};
$refState = $cohortState('rec_cn_ref_0001');
ok($refState['state'] === 'applied' && $refState['status'] === 'recovery_only' && $refState['sequence'] === 1, 'refunded record recovery_only during rehearsal');
$revState = $cohortState('rec_cn_rev_0001');
ok($revState['status'] === 'recovery_only', 'revoked record recovery_only during rehearsal');
okThrows(static fn() => $service->reactivate('rec_cn_ref_0001', ['adverse_state' => 'refunded']), 'REFUNDED', 'refunded record never reactivates during rehearsal');
okThrows(static fn() => $service->reactivate('rec_cn_rev_0001', ['adverse_state' => 'revoked']), 'REVOKED', 'revoked record never reactivates during rehearsal');

// Preservation-only rollback rehearsal: zero rows changed, rehearsal journaled.
$beforeRehearsal = $counts();
$preserved = $schema->preserveForRollback('2026-08-09T02:00:00Z', ['source' => 'final_cutover_canary_rollback_rehearsal']);
ok($preserved['action'] === 'preserve', 'rollback rehearsal is preservation-only');
ok($counts() === $beforeRehearsal, 'rollback rehearsal preserves every canary table');
$schemaEvent = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_schema_events WHERE event_type = 'rollback_preserved'")->fetchColumn();
ok($schemaEvent === 1, 'rollback rehearsal journaled as rollback_preserved');

// New issuance fails closed during the rollback rehearsal: the cutover state
// is intact (never rolled back), so denied routes stay denied and the legacy
// registry stays read-only.
$stillDenied = $cutoverService->routeDisposition($deniedBySurface['install_site_create']['route'], $correlation(6, 'iss'));
ok($stillDenied['disposition'] === 'denied_issuance', 'new issuance still fails closed after rehearsal');
okThrows(static fn() => $cutoverService->legacyTableReadOnlyGate('wpuiai_licenses', 'UPDATE', $correlation(7, 'iss')), 'LEGACY_TABLE_READ_ONLY', 'legacy registry still read-only after rehearsal');
ok($service->journalChainValid() === true && $cutoverService->journalChainValid() === true, 'both journal chains valid after rehearsal');

// Replay: second canary cycle returns stored outcomes with zero new rows.
$replayedStart = $service->startCanary($startInput);
ok($replayedStart['replayed'] === true, 'canary start replay returns stored run');
$replayedRecon = $service->reconcile($reconInput);
ok($replayedRecon['replayed'] === true && $replayedRecon['matching'] === true, 'reconcile replay returns stored result');
$replayedProof = $service->proveRollback($proofInput);
ok($replayedProof['replayed'] === true && $replayedProof['sequence_preserved'] === true, 'rollback proof replay returns stored result');
ok($counts() === $beforeRehearsal, 'replays write zero rows');

$finalCounts = $counts();
$appliedRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_cohort WHERE canary_state = 'applied'")->fetchColumn();
$quarantinedRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_cohort WHERE canary_state = 'quarantined'")->fetchColumn();
$recoveryOnlyRows = (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_canary_cohort WHERE canary_state = 'applied' AND outcome_payload LIKE '%\"status\":\"recovery_only\"%'")->fetchColumn();
$summaryOut = [
    'schema' => 'focusa.spec152e.final_cutover_canary_test.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'cohort_size' => $finalCounts['cohort'],
    'applied_entries' => $appliedRows,
    'quarantined_entries' => $quarantinedRows,
    'ledger_rows' => $finalCounts['ledger'],
    'reconciled_rows' => $finalCounts['reconciliation'],
    'rollback_proof_rows' => $finalCounts['rollback'],
    'recovery_only_rows' => $recoveryOnlyRows,
    'dry_run_writes' => 0,
    'replay_second_rows' => 0,
    'denials_journaled' => $finalCounts['denials'],
    'legacy_tables_read_only' => $finalCounts['legacy_tables'],
    'recovery_surfaces_retained' => $finalCounts['recovery_surfaces'],
    'facade_proxy_actions' => count($cutoverFixture['facade_proxy_routes']),
    'journal_chain_valid' => true,
    'rollback_rehearsal_preservation_only' => true,
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summaryOut, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
"""


def run_harness() -> str:
    if not PHP:
        raise AssertionError("FAIL: php is required to execute the final cutover canary adapter")
    with tempfile.TemporaryDirectory() as tmp:
        harness_path = Path(tmp) / "final_cutover_canary_harness.php"
        harness_path.write_text(HARNESS, encoding="utf-8")
        proc = subprocess.run(
            [PHP, str(harness_path), str(CANARY_CONTRACT), str(CUTOVER_CONTRACT), str(CANARY_FIXTURE), str(CUTOVER_FIXTURE)],
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
expect(result["recovery_only_rows"] == 2, "refunded+revoked recovery_only")
expect(result["dry_run_writes"] == 0, "dry run writes zero rows")
expect(result["replay_second_rows"] == 0, "replays write zero rows")
expect(result["denials_journaled"] == 6, "four install-site + stripe + eval denials journaled once")
expect(result["legacy_tables_read_only"] == 2, "two legacy tables read-only")
expect(result["recovery_surfaces_retained"] == 9, "nine retained recovery surfaces")
expect(result["facade_proxy_actions"] == 11, "eleven facade proxy actions")
expect(result["journal_chain_valid"] is True, "journal chains valid")
expect(result["rollback_rehearsal_preservation_only"] is True, "rollback rehearsal preservation-only")

positive = result["positive_checks"]
negative = result["negative_checks"]

summary = {
    "schema": "focusa.spec152e.final_cutover_canary_validation.v1",
    "canary_fixture_sha256": sha256(canary_fixture_raw),
    "cutover_fixture_sha256": sha256(cutover_fixture_raw),
    "paid_fixture_sha256": sha256(paid_raw),
    "legacy_fixture_sha256": sha256(legacy_raw),
    "inventory_fixture_sha256": sha256(inventory_raw),
    "facade_fixture_sha256": sha256(facade_raw),
    "product_fixture_sha256": sha256(product_raw),
    "canary_contract_sha256": sha256(canary_raw),
    "cutover_contract_sha256": sha256(cutover_raw),
    "harness_sha256": sha256(first),
    "positive_checks": positive,
    "negative_checks": negative,
    "cohort_size": result["cohort_size"],
    "applied_entries": result["applied_entries"],
    "quarantined_entries": result["quarantined_entries"],
    "ledger_rows": result["ledger_rows"],
    "reconciled_rows": result["reconciled_rows"],
    "rollback_proof_rows": result["rollback_proof_rows"],
    "recovery_only_rows": result["recovery_only_rows"],
    "dry_run_writes": result["dry_run_writes"],
    "replay_second_rows": result["replay_second_rows"],
    "denials_journaled": result["denials_journaled"],
    "legacy_tables_read_only": result["legacy_tables_read_only"],
    "recovery_surfaces_retained": result["recovery_surfaces_retained"],
    "facade_proxy_actions": result["facade_proxy_actions"],
    "journal_chain_valid": result["journal_chain_valid"],
    "rollback_rehearsal_preservation_only": result["rollback_rehearsal_preservation_only"],
    "result": "passed",
}
print(json.dumps(summary, sort_keys=True))
