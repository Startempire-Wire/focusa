#!/usr/bin/env python3
"""Spec 152E.07.05 UIAI and bundle account/product isolation e2e gate
(atom focusa-vbcqu.20.13.59).

Exact verification:
    python3 tests/spec152e_uiai_bundle_e2e_test.py

Runs seven synthetic test-mode journeys through one deterministic PHP harness
on sqlite (spec 152E §7, §8, §11, §15, §16, §18, §23 rows "UIAI purchase",
"Bundle purchase", "Wrong product"; Spec 158 implementation excluded; Spec 172
overlay binding: dedicated Downloads offer, UIAI local families, frozen
hosted-resource exclusions). The harness output is byte-identical across runs
(replayable from the pinned commit):

- Focusa-only (session A): verified email -> EDD order 9616 -> EDD Software
  Licensing key 7781 -> dual delivery (email + account, same canonical key) ->
  node -> signed lease (sequence 1, grants exactly focusa) -> UIAI child
  token DENIED (CHILD_TOKEN_NOT_INCLUDED) -> refund (sequence 2,
  recovery_only) -> reactivation DENIED (REFUNDED_NEVER_REACTIVATES).
- UIAI-only (session B): same flow -> lease (grants exactly uiai_engine) ->
  bounded child token (15-minute TTL, exact UIAI family subset, digest-only
  at rest) -> refund.
- Bundle (session C): ONE composite SKU at 1254.60 -> ONE canonical human key
  -> signed lease (sequence 1, posture bundle, grants = exact union of both
  Operator v1 records, 13-family derived union) -> bounded child token from
  the bundle's UIAI grant -> refund. One verified account, no silent partial
  success.
- Shared identity (session D): the SAME verified email identity as the Bundle
  customer buys Focusa; promotion REUSES the same authority account and the
  same EDD customer id (identity_reused=true, zero new rows) with an
  independent per-product sequence ledger.
- Partial delivery (session E): a test-mode email-channel bounce marks
  delivery partial (account channel sent) — node registration and lease
  issuance fail closed (LICENSE_DELIVERY_PENDING / NODE_REQUIRED), a second
  deliver fails DELIVERY_ALREADY_PARTIAL, recoverDelivery() retries ONLY the
  failed channel and never duplicates the key, then node/lease/refund.
- Reactivation (session G): refunded purchase 1 never reactivates
  (REFUNDED_NEVER_REACTIVATES); purchase 2 by the same verified identity is a
  NEW EDD order/license/lease at monotonic sequence 3 while the refunded
  rows stay preserved.
- Wrong product: caller product/price/grant fields fail closed
  (CALLER_CONTROLLED_GRANT_DENIED), unknown codes fail
  (PRODUCT_MAPPING_REQUIRED), resolveProductGrants() never expands a wrong
  product, and no cross-product lease or downgrade exists.

Python independently re-verifies every signed lease with `cryptography`
Ed25519PublicKey over the domain-separated payload (byte-compatible with the
Rust verifier), derives the expected canonical EDD human key from the pinned
license/order vectors and matches the delivered key masks, and verifies each
child token's bounded TTL, exact feature subset, limits, and digest-only
storage. Secrets and unmasked real email are absent from every artifact; the
immutable receipt sha256 handles are recorded.

Surfaces under test (EXACT SURFACES):
- UIAI and bundle EDD products + shared identity:
  docs/contracts/spec152e-uiai-bundle-isolation.v1.php (startRegistration,
  verifyEmail, promote, createCheckoutIntent, completePayment, issueLicense,
  deliver, recoverDelivery, registerNode, issueLease, issueChildToken,
  refund, reactivate, resolveProductGrants, receipt) + fixture
  docs/contracts/spec152e-uiai-bundle-isolation-fixture.v1.json
- Key/lease delivery: the canonical EDD key through email + account channels
  with the same masked key; the signed device-bound lease via the canonical
  Ed25519 key-set seam from docs/contracts/spec152e-edd-bound-lease-issuer.v1.php
  (loaded first) and the frozen UIAI hosted-resource exclusion registry from
  docs/contracts/spec172-uiai-hosted-resource-exclusion-registry.v1.php.

Fail-closed invariants (spec 152E FORBIDDEN + §19):
- No unverified-email promotion, no local/self-issued entitlement, no
  independent facade authority, no client-controlled EDD price/grants.
- Each purchase grants EXACT products only; the Bundle uses ONE verified
  account and ONE human key with no silent partial success.
- No raw email, raw key, payment reference, token secret, or credential
  material in any artifact; child tokens are digest-only at rest; receipts are
  masked and carry an immutable sha256 handle.
- No push, deploy, release, merge, or Beads mutation is performed.
"""

import base64
import datetime as _dt
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec152e-uiai-bundle-isolation.v1.php"
LEASE_CONTRACT = ROOT / "docs/contracts/spec152e-edd-bound-lease-issuer.v1.php"
PROJECTOR_CONTRACT = ROOT / "docs/contracts/spec172-edd-license-type-projector.v1.php"
EXCLUSION_CONTRACT = ROOT / "docs/contracts/spec172-uiai-hosted-resource-exclusion-registry.v1.php"
FIXTURE = ROOT / "docs/contracts/spec152e-uiai-bundle-isolation-fixture.v1.json"

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


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def derive_expected_key(license_id: int, order_id: int) -> str:
    """Deterministic canonical EDD Software Licensing human key (same derivation
    as the PHP contract): 'FOCUSA-' + 4x4 uppercase hex groups."""
    raw = hashlib.sha256(f"edd-sl-v1\n{license_id}\n{order_id}".encode()).hexdigest().upper()
    raw = re.sub(r"[^A-Z0-9]", "", raw)[:16]
    return "FOCUSA-" + "-".join(raw[i:i + 4] for i in range(0, 16, 4))


# ── Deterministic PHP journey harness ───────────────────────────────────────

HARNESS = r"""<?php
// Spec 152E.07.05 UIAI + bundle account/product isolation journey harness
// (generated by the python gate). Executes the seven deterministic test-mode
// journeys on sqlite with a fixed clock and the canonical Ed25519 authority
// key-set seam, then emits a deterministic redacted summary. Byte-identical
// across runs; every positive/negative check is counted. No raw email, raw
// key, payment reference, token secret, or credential material ever appears
// in the summary.
declare(strict_types=1);
$leaseContractPath = $argv[1];
$projectorContractPath = $argv[2];
$exclusionContractPath = $argv[3];
$contractPath = $argv[4];
$fixturePath = $argv[5];
require_once $leaseContractPath;
require_once $projectorContractPath;
require_once $exclusionContractPath;
require_once $contractPath;
$fixture = json_decode((string) file_get_contents($fixturePath), true, 512, JSON_THROW_ON_ERROR);
$positive = 0;
$negative = 0;
function ok(bool $condition, string $message): void { global $positive; $positive++; if (!$condition) { fwrite(STDERR, "FAIL: {$message}\n"); exit(1); } }
function okThrows(callable $operation, string $code, string $message): void { global $negative; $negative++; try { $operation(); } catch (DomainException $error) { if ($error->getMessage() === $code) { return; } fwrite(STDERR, "FAIL: {$message} (got {$error->getMessage()})\n"); exit(1); } catch (Throwable $error) { fwrite(STDERR, "FAIL: {$message} (unexpected " . get_class($error) . ": " . $error->getMessage() . ")\n"); exit(1); } fwrite(STDERR, "FAIL: {$message} (no throw)\n"); exit(1); }

$db = new PDO('sqlite::memory:');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
$tick = 0;
$clock = static function () use (&$tick): string {
    $ts = (new DateTimeImmutable('2026-08-09T07:00:00Z'))->modify('+' . $tick . ' minutes')->format('Y-m-d\TH:i:s\Z');
    $tick++;
    return $ts;
};
$schema = new FocusaSpec152eUiaiBundleIsolationMigration($db, 'wp_');
$schema->migrate('2026-08-09T06:00:00Z', ['source' => 'uiai_bundle_isolation_test']);
$keySet = new FocusaSpec152eAuthorityKeySetSeam(str_repeat('R', 32), str_repeat('L', 32), $clock);
$svc = new FocusaSpec152eUiaiBundleIsolationService($db, $clock, 'wp_', $keySet);

$counts = static function () use ($db): array {
    $table = static fn(string $name): int => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_ubi_{$name}")->fetchColumn();
    return [
        'registrations' => $table('registrations'),
        'identities' => $table('identities'),
        'accounts' => $table('accounts'),
        'orders' => $table('orders'),
        'order_items' => $table('order_items'),
        'licenses' => $table('licenses'),
        'deliveries' => $table('deliveries'),
        'nodes' => $table('nodes'),
        'leases' => $table('leases'),
        'sequences' => $table('sequences'),
        'refunds' => $table('refunds'),
        'child_tokens' => $table('child_tokens'),
        'journal' => $table('journal'),
    ];
};
$correlation = static function (int $seq, string $kind): array {
    return [
        'request_id' => 'req_ubi_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'idempotency_key' => 'idem_ubi_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
    ];
};
$journeys = [];
$focusa = $fixture['sessions']['focusa'];
$uiai = $fixture['sessions']['uiai'];
$bundle = $fixture['sessions']['bundle'];
$shared = $fixture['sessions']['shared'];
$partial = $fixture['sessions']['partial'];
$react = $fixture['sessions']['reactivation'];
$facade = $fixture['facade'];
$products = $fixture['products'];
$leaseCfg = $fixture['lease'];
$childCfg = $fixture['child_token'];

// ── Pre-flight negatives: no operation works without a verified registration ──
$bogus = '00000000-0000-0000-0000-000000000000';
okThrows(static fn() => $svc->verifyEmail(array_merge(['registration_uuid' => $bogus, 'code' => $focusa['verification_code']], $correlation(1, 'pre'))), 'REGISTRATION_NOT_FOUND', 'verify on unknown registration');
okThrows(static fn() => $svc->receipt(['registration_uuid' => $bogus]), 'REGISTRATION_NOT_FOUND', 'receipt on unknown registration');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => 'attacker_v1', 'origin' => 'https://evil.invalid', 'product_code' => $focusa['product_code'], 'email_digest' => $focusa['email_digest'], 'email_domain' => $focusa['email_domain'], 'email_prefix_char' => 'x', 'challenge_code' => $focusa['verification_code']], $correlation(1, 'pre'))), 'FACADE_ORIGIN_DENIED', 'facade spoof denied');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $facade['facade_id'], 'origin' => $facade['origin'], 'product_code' => 'focusa_hacker', 'email_digest' => $focusa['email_digest'], 'email_domain' => $focusa['email_domain'], 'email_prefix_char' => 'x', 'challenge_code' => $focusa['verification_code']], $correlation(1, 'pre'))), 'PRODUCT_MAPPING_REQUIRED', 'unmapped product denied');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $facade['facade_id'], 'origin' => $facade['origin'], 'product_code' => $focusa['product_code'], 'email_digest' => $focusa['email_digest'], 'email_domain' => $focusa['email_domain'], 'email_prefix_char' => 'x', 'challenge_code' => $focusa['verification_code'], 'price' => '0'], $correlation(1, 'pre'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller price denied at registration');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $facade['facade_id'], 'origin' => $facade['origin'], 'product_code' => $focusa['product_code'], 'email_digest' => $focusa['email_digest'], 'email_domain' => $focusa['email_domain'], 'email_prefix_char' => 'x', 'challenge_code' => $focusa['verification_code'], 'grants' => ['uiai_engine']], $correlation(1, 'pre'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller grants denied at registration');
okThrows(static fn() => $svc->resolveProductGrants(['product_code' => 'focusa_hacker']), 'PRODUCT_MAPPING_REQUIRED', 'unknown product code denied by the wrong-product guard');

// ── Session A: Focusa-only purchase grants Focusa ONLY ─────────────────────
$startA = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $focusa['product_code'],
    'email_digest' => $focusa['email_digest'], 'email_domain' => $focusa['email_domain'],
    'email_prefix_char' => $focusa['email_prefix_char'], 'challenge_code' => $focusa['verification_code'],
], $correlation(2, 'start')));
ok($startA['ok'] === true && $startA['state'] === 'attempt_created', 'focusa registration created');
ok($startA['masked_email'] === 'f***@invalid.example', 'focusa registration masks the email');
ok($startA['customer_created'] === false && $startA['entitlement_created'] === false, 'focusa registration creates no customer/entitlement');
$regA = $startA['registration_uuid'];
$journeys['focusa'][] = $startA['state'];
$replayA = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $focusa['product_code'],
    'email_digest' => $focusa['email_digest'], 'email_domain' => $focusa['email_domain'],
    'email_prefix_char' => $focusa['email_prefix_char'], 'challenge_code' => $focusa['verification_code'],
], $correlation(2, 'start')));
ok($replayA['replayed'] === true && $replayA['idempotent_replay'] === true, 'focusa registration replay is idempotent');
okThrows(static fn() => $svc->verifyEmail(array_merge(['registration_uuid' => $regA, 'code' => '000000'], $correlation(3, 'verify'))), 'EMAIL_VERIFICATION_FAILED', 'wrong verification code fails');
$verifiedA = $svc->verifyEmail(array_merge(['registration_uuid' => $regA, 'code' => $focusa['verification_code']], $correlation(4, 'verify')));
ok($verifiedA['ok'] === true && $verifiedA['state'] === 'email_verified', 'focusa mailbox verified');
$journeys['focusa'][] = $verifiedA['state'];
okThrows(static fn() => $svc->verifyEmail(array_merge(['registration_uuid' => $regA, 'code' => $focusa['verification_code']], $correlation(5, 'verify'))), 'EMAIL_VERIFICATION_FAILED', 'focusa verification code is single-use');
$promotedA = $svc->promote(array_merge(['registration_uuid' => $regA], $correlation(4, 'promote')));
ok($promotedA['ok'] === true && $promotedA['state'] === 'account_promoted', 'focusa account promoted');
ok((int) $promotedA['customer_id'] === (int) $focusa['customer_id'], 'focusa EDD customer 2052 created');
ok($promotedA['identity_reused'] === false && $promotedA['zero_new_rows'] === false, 'focusa promotion creates the first identity/account');
$accountA = $promotedA['account_uuid'];
$journeys['focusa'][] = $promotedA['state'];
$replayPromoteA = $svc->promote(array_merge(['registration_uuid' => $regA], $correlation(4, 'promote')));
ok($replayPromoteA['replayed'] === true && $replayPromoteA['zero_new_rows'] === true, 'focusa promote replay writes zero rows');
okThrows(static fn() => $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regA, 'product_code' => 'focusa_hacker'], $correlation(6, 'intent'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller product denied at checkout');
okThrows(static fn() => $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regA, 'price' => '0'], $correlation(6, 'intent'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller price denied at checkout');
$intentA = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regA], $correlation(7, 'intent')));
ok($intentA['ok'] === true && $intentA['state'] === 'checkout_pending', 'focusa checkout intent created');
ok(str_starts_with($intentA['branded_checkout_url'], $facade['origin'] . $facade['checkout_path']), 'focusa branded facade checkout URL');
ok((int) $intentA['edd_download_id'] === (int) $products[$focusa['product_code']]['edd_download_id'] && $intentA['edd_price_id'] === $products[$focusa['product_code']]['edd_price_id'] && $intentA['price_usd'] === $products[$focusa['product_code']]['price_usd'], 'focusa server-owned product/price mapping');
ok($intentA['grants'] === ['focusa_operator_lifetime_v1'] && $intentA['products'] === ['focusa'], 'focusa checkout grants exactly the Focusa product');
ok($intentA['stripe_gateway'] === 'edd_stripe_test_mode' && $intentA['card_data_handled_by'] === 'edd_stripe_only', 'EDD Stripe gateway only, no client card data');
$journeys['focusa'][] = $intentA['state'];
$heldA = $svc->completePayment(array_merge([
    'registration_uuid' => $regA, 'checkout_email_digest' => $focusa['invalid_email_digest'],
    'payment_reference_digest' => $focusa['payment_reference_digest'],
], $correlation(8, 'pay')));
ok($heldA['ok'] === false && $heldA['state'] === 'held_unverified' && $heldA['error'] === 'EDD_ORDER_UNVERIFIED', 'different checkout email holds focusa fulfillment');
ok($heldA['checkout_email_integrity'] === 'fulfillment_held', 'payment success alone cannot bypass verification');
okThrows(static fn() => $svc->issueLicense(array_merge(['registration_uuid' => $regA], $correlation(8, 'license'))), 'EDD_ORDER_PENDING', 'no focusa license while fulfillment is held');
$paidA = $svc->completePayment(array_merge([
    'registration_uuid' => $regA, 'checkout_email_digest' => $focusa['email_digest'],
    'payment_reference_digest' => $focusa['payment_reference_digest'],
], $correlation(9, 'pay')));
ok($paidA['ok'] === true && $paidA['state'] === 'complete', 'focusa EDD order completed');
ok((int) $paidA['order_id'] === (int) $focusa['order_id'] && $paidA['checkout_email_integrity'] === 'verified_identity_match', 'one canonical focusa order, verified identity match');
$journeys['focusa'][] = 'order_complete';
$licenseA = $svc->issueLicense(array_merge(['registration_uuid' => $regA], $correlation(9, 'license')));
ok($licenseA['ok'] === true && $licenseA['state'] === 'entitlement_issued', 'focusa EDD Software Licensing key issued');
ok((int) $licenseA['edd_license_id'] === (int) $focusa['edd_license_id'] && $licenseA['source'] === 'edd_software_licensing', 'focusa key source is EDD only');
ok($licenseA['issuance_surface'] === 'edd_authority_only' && $licenseA['duplicate_license'] === false, 'no local/self-issued focusa entitlement');
ok($licenseA['grants'] === ['focusa_operator_lifetime_v1'] && $licenseA['human_key_count'] === 1, 'focusa license grants exactly one product, one key');
$maskA = $licenseA['license_key_mask'];
ok(preg_match('/^FOCUSA-[A-Z0-9]{4}-\*{4}-\*{4}-\*{4}$/', $maskA) === 1, 'focusa issuance returns a masked key only');
$journeys['focusa'][] = $licenseA['state'];
$deliveryA = $svc->deliver(array_merge(['registration_uuid' => $regA], $correlation(10, 'deliver')));
ok($deliveryA['ok'] === true && $deliveryA['state'] === 'delivered', 'focusa key delivered');
ok($deliveryA['channels'] === ['email' => 'sent', 'account' => 'sent'], 'focusa dual-channel delivery (email + account)');
ok($deliveryA['same_canonical_key_both_channels'] === true && $deliveryA['promotional_content'] === false, 'focusa same canonical key both channels; transactional only');
ok($deliveryA['key_mask'] === $maskA, 'focusa delivery masks the same canonical key');
$journeys['focusa'][] = $deliveryA['state'];
$nodeA = $svc->registerNode(array_merge(['registration_uuid' => $regA, 'node_id' => $focusa['node_id'], 'device_public_key' => $focusa['device_public_key_b64']], $correlation(11, 'node')));
ok($nodeA['ok'] === true && $nodeA['state'] === 'device_registered', 'focusa node registered');
ok($nodeA['binding'] === 'account_and_edd_license' && $nodeA['install_channel_telemetry_only'] === true, 'focusa node bound to account + EDD license; channel telemetry only');
$journeys['focusa'][] = $nodeA['state'];
okThrows(static fn() => $svc->issueLease(array_merge(['registration_uuid' => $regA, 'features' => ['all' => true]], $correlation(12, 'lease'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller grants denied at lease');
$leaseA = $svc->issueLease(array_merge(['registration_uuid' => $regA], $correlation(12, 'lease')));
ok($leaseA['ok'] === true && $leaseA['state'] === 'lease_issued', 'focusa signed lease issued');
ok((int) $leaseA['sequence'] === (int) $leaseCfg['sequence_after_lease'] && $leaseA['posture'] === $leaseCfg['posture_paid'], 'focusa lease sequence 1, paid posture');
ok($leaseA['authority_key_id'] === $leaseCfg['lease_key_id'] && $leaseA['runtime_authorization'] === 'signed_device_bound_lease', 'focusa authority lease key, device-bound runtime authorization');
$journeys['focusa'][] = $leaseA['state'];
okThrows(static fn() => $svc->reactivate(array_merge(['registration_uuid' => $regA], $correlation(12, 'reactivate-pre'))), 'REACTIVATION_REQUIRES_NEW_ORDER', 'reactivating a non-refunded registration requires a new order');
$leaseRowA = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseA['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
// Focusa-only purchase can NEVER derive a UIAI child token.
okThrows(static fn() => $svc->issueChildToken(array_merge(['registration_uuid' => $regA], $correlation(13, 'child'))), 'CHILD_TOKEN_NOT_INCLUDED', 'focusa-only purchase is denied the UIAI child token');
// Wrong-product guard: the Focusa grant is exact and never contains UIAI.
$focusaGrants = $svc->resolveProductGrants(['product_code' => $focusa['product_code']]);
ok($focusaGrants['products'] === ['focusa'] && $focusaGrants['grants'] === ['focusa_operator_lifetime_v1'], 'focusa grants resolve exactly, never UIAI');
$uiaiGrants = $svc->resolveProductGrants(['product_code' => $uiai['product_code']]);
ok($uiaiGrants['products'] === ['uiai_engine'] && $uiaiGrants['grants'] === ['uiai_operator_lifetime_v1'], 'uiai grants resolve exactly, never Focusa');
$bundleGrants = $svc->resolveProductGrants(['product_code' => $bundle['product_code']]);
ok($bundleGrants['products'] === ['focusa', 'uiai_engine'] && $bundleGrants['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'bundle grants resolve to the exact union only');
ok($bundleGrants['grant_composition'] === 'exact_union' && $bundleGrants['human_key_count'] === 1, 'bundle is one SKU, one key, exact union');
$refundA = $svc->refund(array_merge(['registration_uuid' => $regA, 'reason' => 'synthetic_proof_cleanup'], $correlation(13, 'refund')));
ok($refundA['ok'] === true && $refundA['state'] === 'refunded', 'focusa refund processed');
ok((int) $refundA['sequence_after'] === (int) $leaseCfg['sequence_after_refund'] && $refundA['refresh_denied'] === true, 'focusa refund increments sequence to 2 and denies refresh');
ok($refundA['posture'] === 'recovery_only' && $refundA['account_order_delivery_node_evidence_preserved'] === true, 'focusa recovery-only posture, truth preserved');
$journeys['focusa'][] = $refundA['state'];
$reactA = $svc->reactivate(array_merge(['registration_uuid' => $regA], $correlation(14, 'reactivate')));
ok($reactA['ok'] === false && $reactA['error'] === 'REFUNDED_NEVER_REACTIVATES', 'focusa refunded record never reactivates');
ok($reactA['reactivation_requires_new_purchase'] === true && $reactA['refresh_denied'] === true, 'focusa reactivation requires a new verified purchase');
$receiptA = $svc->receipt(['registration_uuid' => $regA]);
ok($receiptA['schema'] === $fixture['receipt']['schema'] && $receiptA['state'] === 'refunded', 'focusa receipt schema and state');
ok($receiptA['install_site_authority'] === 'none' && $receiptA['spec158'] === 'excluded', 'no install-site authority; spec 158 excluded');
ok($receiptA['masked_email'] === 'f***@invalid.example' && $receiptA['grants'] === ['focusa_operator_lifetime_v1'], 'focusa receipt masked email and exact grants');
ok((int) $receiptA['lease_sequence'] === 2 && $receiptA['lease_state'] === 'refunded', 'focusa receipt shows revoked lease and sequence 2');
ok(preg_match('/^[0-9a-f]{64}$/', (string) $receiptA['receipt_sha256']) === 1, 'focusa receipt carries an immutable sha256 handle');

// ── Session B: UIAI-only purchase grants UIAI ONLY + bounded child token ────
$startB = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $uiai['product_code'],
    'email_digest' => $uiai['email_digest'], 'email_domain' => $uiai['email_domain'],
    'email_prefix_char' => $uiai['email_prefix_char'], 'challenge_code' => $uiai['verification_code'],
], $correlation(20, 'start')));
ok($startB['ok'] === true && $startB['state'] === 'attempt_created', 'uiai registration created');
$regB = $startB['registration_uuid'];
$journeys['uiai'][] = $startB['state'];
$verifiedB = $svc->verifyEmail(array_merge(['registration_uuid' => $regB, 'code' => $uiai['verification_code']], $correlation(20, 'verify')));
ok($verifiedB['ok'] === true, 'uiai mailbox verified');
$journeys['uiai'][] = $verifiedB['state'];
$promotedB = $svc->promote(array_merge(['registration_uuid' => $regB], $correlation(20, 'promote')));
ok((int) $promotedB['customer_id'] === (int) $uiai['customer_id'], 'uiai EDD customer 2380 created');
$accountB = $promotedB['account_uuid'];
ok($accountB !== $accountA, 'uiai purchase uses its own independent account');
$journeys['uiai'][] = $promotedB['state'];
$intentB = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regB], $correlation(21, 'intent')));
ok($intentB['ok'] === true && $intentB['price_usd'] === $products[$uiai['product_code']]['price_usd'], 'uiai checkout intent at the uiai price');
ok($intentB['grants'] === ['uiai_operator_lifetime_v1'] && $intentB['products'] === ['uiai_engine'], 'uiai checkout grants exactly the UIAI product');
$journeys['uiai'][] = $intentB['state'];
$heldB = $svc->completePayment(array_merge([
    'registration_uuid' => $regB, 'checkout_email_digest' => $uiai['invalid_email_digest'],
    'payment_reference_digest' => $uiai['payment_reference_digest'],
], $correlation(22, 'pay')));
ok($heldB['state'] === 'held_unverified' && $heldB['error'] === 'EDD_ORDER_UNVERIFIED', 'uiai checkout email mismatch holds fulfillment');
$paidB = $svc->completePayment(array_merge([
    'registration_uuid' => $regB, 'checkout_email_digest' => $uiai['email_digest'],
    'payment_reference_digest' => $uiai['payment_reference_digest'],
], $correlation(23, 'pay')));
ok($paidB['ok'] === true && (int) $paidB['order_id'] === (int) $uiai['order_id'], 'uiai EDD order completed');
$journeys['uiai'][] = 'order_complete';
$licenseB = $svc->issueLicense(array_merge(['registration_uuid' => $regB], $correlation(23, 'license')));
ok($licenseB['ok'] === true && (int) $licenseB['edd_license_id'] === (int) $uiai['edd_license_id'], 'uiai EDD key issued');
ok($licenseB['grants'] === ['uiai_operator_lifetime_v1'] && $licenseB['human_key_count'] === 1, 'uiai license grants exactly the UIAI product, one key');
$maskB = $licenseB['license_key_mask'];
$journeys['uiai'][] = $licenseB['state'];
$deliveryB = $svc->deliver(array_merge(['registration_uuid' => $regB], $correlation(24, 'deliver')));
ok($deliveryB['channels'] === ['email' => 'sent', 'account' => 'sent'] && $deliveryB['key_mask'] === $maskB, 'uiai dual-channel delivery, same canonical key');
$journeys['uiai'][] = $deliveryB['state'];
$nodeB = $svc->registerNode(array_merge(['registration_uuid' => $regB, 'node_id' => $uiai['node_id'], 'device_public_key' => $uiai['device_public_key_b64']], $correlation(25, 'node')));
ok($nodeB['ok'] === true, 'uiai node registered');
$journeys['uiai'][] = $nodeB['state'];
$leaseB = $svc->issueLease(array_merge(['registration_uuid' => $regB], $correlation(25, 'lease')));
ok($leaseB['ok'] === true && (int) $leaseB['sequence'] === 1 && $leaseB['posture'] === $leaseCfg['posture_paid'], 'uiai signed lease issued, sequence 1');
$journeys['uiai'][] = $leaseB['state'];
$leaseRowB = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseB['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
$ctB = $svc->issueChildToken(array_merge(['registration_uuid' => $regB], $correlation(26, 'child')));
ok($ctB['ok'] === true && $ctB['state'] === 'child_token_issued', 'uiai bounded child token issued');
$tokenB = $ctB['child_token'];
ok($tokenB['schema'] === $childCfg['schema'] && $tokenB['audience'] === $childCfg['audience'], 'uiai child token schema and audience');
ok($tokenB['features'] === $childCfg['features'], 'uiai child token features are exactly the UIAI local families');
ok($tokenB['limits'] === $childCfg['limits'], 'uiai child token limits are the frozen seat/node limits');
ok(preg_match('/^[0-9a-f]{64}$/', (string) $ctB['token_digest']) === 1 && $ctB['token_stored'] === 'digest_only', 'uiai child token is digest-only at rest');
ok((int) $ctB['max_ttl_minutes'] === (int) $childCfg['max_ttl_minutes'], 'uiai child token TTL is bounded to 15 minutes');
$journeys['uiai'][] = 'child_token_issued';
$refundB = $svc->refund(array_merge(['registration_uuid' => $regB, 'reason' => 'synthetic_proof_cleanup'], $correlation(26, 'refund')));
ok($refundB['ok'] === true && (int) $refundB['sequence_after'] === 2, 'uiai refund increments sequence to 2');
$journeys['uiai'][] = $refundB['state'];
$receiptB = $svc->receipt(['registration_uuid' => $regB]);
ok($receiptB['state'] === 'refunded' && $receiptB['grants'] === ['uiai_operator_lifetime_v1'], 'uiai receipt redacted with exact grants');
ok(preg_match('/^[0-9a-f]{64}$/', (string) $receiptB['receipt_sha256']) === 1, 'uiai receipt immutable handle');

// ── Session C: Bundle = ONE SKU, ONE key, exact union, ONE verified account ──
$startC = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $bundle['product_code'],
    'email_digest' => $bundle['email_digest'], 'email_domain' => $bundle['email_domain'],
    'email_prefix_char' => $bundle['email_prefix_char'], 'challenge_code' => $bundle['verification_code'],
], $correlation(30, 'start')));
ok($startC['ok'] === true && $startC['state'] === 'attempt_created', 'bundle registration created');
$regC = $startC['registration_uuid'];
$journeys['bundle'][] = $startC['state'];
$verifiedC = $svc->verifyEmail(array_merge(['registration_uuid' => $regC, 'code' => $bundle['verification_code']], $correlation(30, 'verify')));
ok($verifiedC['ok'] === true, 'bundle mailbox verified');
$journeys['bundle'][] = $verifiedC['state'];
$promotedC = $svc->promote(array_merge(['registration_uuid' => $regC], $correlation(30, 'promote')));
ok((int) $promotedC['customer_id'] === (int) $bundle['customer_id'], 'bundle EDD customer 2562 created');
$accountC = $promotedC['account_uuid'];
$journeys['bundle'][] = $promotedC['state'];
$intentC = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regC], $correlation(31, 'intent')));
ok($intentC['ok'] === true && $intentC['price_usd'] === $products[$bundle['product_code']]['price_usd'] && $intentC['price_usd'] === '1254.60', 'bundle checkout at the canonical 1254.60 price');
ok($intentC['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'] && $intentC['grant_composition'] === 'exact_union', 'bundle checkout carries the exact two-product union');
$journeys['bundle'][] = $intentC['state'];
$heldC = $svc->completePayment(array_merge([
    'registration_uuid' => $regC, 'checkout_email_digest' => $bundle['invalid_email_digest'],
    'payment_reference_digest' => $bundle['payment_reference_digest'],
], $correlation(32, 'pay')));
ok($heldC['state'] === 'held_unverified', 'bundle checkout email mismatch holds fulfillment');
$paidC = $svc->completePayment(array_merge([
    'registration_uuid' => $regC, 'checkout_email_digest' => $bundle['email_digest'],
    'payment_reference_digest' => $bundle['payment_reference_digest'],
], $correlation(33, 'pay')));
ok($paidC['ok'] === true && (int) $paidC['order_id'] === (int) $bundle['order_id'], 'bundle EDD order completed');
$journeys['bundle'][] = 'order_complete';
$licenseC = $svc->issueLicense(array_merge(['registration_uuid' => $regC], $correlation(33, 'license')));
ok($licenseC['ok'] === true && (int) $licenseC['edd_license_id'] === (int) $bundle['edd_license_id'], 'bundle issues ONE canonical EDD key');
ok($licenseC['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'] && $licenseC['grant_composition'] === 'exact_union', 'bundle key grants the exact union of both Operator records');
ok($licenseC['human_key_count'] === 1 && $licenseC['duplicate_license'] === false, 'bundle uses exactly one human key, no duplicates');
$maskC = $licenseC['license_key_mask'];
$journeys['bundle'][] = $licenseC['state'];
$deliveryC = $svc->deliver(array_merge(['registration_uuid' => $regC], $correlation(34, 'deliver')));
ok($deliveryC['channels'] === ['email' => 'sent', 'account' => 'sent'] && $deliveryC['key_mask'] === $maskC, 'bundle dual-channel delivery, same canonical key');
$journeys['bundle'][] = $deliveryC['state'];
$nodeC = $svc->registerNode(array_merge(['registration_uuid' => $regC, 'node_id' => $bundle['node_id'], 'device_public_key' => $bundle['device_public_key_b64']], $correlation(35, 'node')));
ok($nodeC['ok'] === true, 'bundle node registered');
$journeys['bundle'][] = $nodeC['state'];
$leaseC = $svc->issueLease(array_merge(['registration_uuid' => $regC], $correlation(35, 'lease')));
ok($leaseC['ok'] === true && (int) $leaseC['sequence'] === 1 && $leaseC['posture'] === $leaseCfg['posture_bundle'], 'bundle signed lease issued, sequence 1, bundle posture');
$journeys['bundle'][] = $leaseC['state'];
$leaseRowC = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseC['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
// The bundle's UIAI grant derives the bounded child token; the bundle never
// doubles the account or the human key.
$ctC = $svc->issueChildToken(array_merge(['registration_uuid' => $regC], $correlation(36, 'child')));
ok($ctC['ok'] === true && $ctC['child_token']['schema'] === $childCfg['schema'], 'bundle derives the bounded UIAI child token');
ok($ctC['child_token']['features'] === $childCfg['features'], 'bundle child token features are exactly the UIAI families (exact subset)');
ok((int) $ctC['max_ttl_minutes'] === 15 && $ctC['token_stored'] === 'digest_only', 'bundle child token bounded TTL and digest-only storage');
$journeys['bundle'][] = 'child_token_issued';
$refundC = $svc->refund(array_merge(['registration_uuid' => $regC, 'reason' => 'synthetic_proof_cleanup'], $correlation(36, 'refund')));
ok($refundC['ok'] === true && (int) $refundC['sequence_after'] === 2, 'bundle refund increments sequence to 2');
$journeys['bundle'][] = $refundC['state'];
$receiptC = $svc->receipt(['registration_uuid' => $regC]);
ok($receiptC['state'] === 'refunded' && $receiptC['grants'] === ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1'], 'bundle receipt redacted with the exact union grants');
ok(preg_match('/^[0-9a-f]{64}$/', (string) $receiptC['receipt_sha256']) === 1, 'bundle receipt immutable handle');
// Exactly one account and one license exist for the bundle identity.
$bundleRows = $db->query("SELECT COUNT(*) FROM wp_wpuiai_ubi_accounts WHERE account_uuid = '{$accountC}'")->fetchColumn();
ok((int) $bundleRows === 1, 'bundle uses exactly one verified account');
$bundleLicenseRows = $db->query("SELECT COUNT(*) FROM wp_wpuiai_ubi_licenses WHERE customer_id = " . (int) $bundle['customer_id'])->fetchColumn();
ok((int) $bundleLicenseRows === 1, 'bundle identity holds exactly one human key');

// ── Session D: shared identity — the bundle customer buys Focusa ───────────
// The SAME verified email identity reuses the SAME account and EDD customer.
$startD = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $shared['product_code'],
    'email_digest' => $shared['email_digest'], 'email_domain' => $shared['email_domain'],
    'email_prefix_char' => $shared['email_prefix_char'], 'challenge_code' => $shared['verification_code'],
], $correlation(40, 'start')));
ok($startD['ok'] === true, 'shared-identity focusa registration created (same email as bundle)');
$regD = $startD['registration_uuid'];
$journeys['shared'][] = $startD['state'];
$verifiedD = $svc->verifyEmail(array_merge(['registration_uuid' => $regD, 'code' => $shared['verification_code']], $correlation(40, 'verify')));
ok($verifiedD['ok'] === true, 'shared-identity mailbox verified');
$journeys['shared'][] = $verifiedD['state'];
$promotedD = $svc->promote(array_merge(['registration_uuid' => $regD], $correlation(40, 'promote')));
ok($promotedD['ok'] === true && (int) $promotedD['customer_id'] === (int) $bundle['customer_id'], 'shared identity reuses the bundle EDD customer 2562');
ok($promotedD['account_uuid'] === $accountC && $promotedD['identity_reused'] === true, 'shared identity reuses the bundle authority account');
ok($promotedD['zero_new_rows'] === true, 'shared identity promotion creates zero new rows');
$journeys['shared'][] = $promotedD['state'];
$intentD = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regD], $correlation(41, 'intent')));
ok($intentD['ok'] === true && $intentD['price_usd'] === '697.00', 'shared-identity focusa checkout at the focusa price');
$journeys['shared'][] = $intentD['state'];
$paidD = $svc->completePayment(array_merge([
    'registration_uuid' => $regD, 'checkout_email_digest' => $shared['email_digest'],
    'payment_reference_digest' => $shared['payment_reference_digest'],
], $correlation(41, 'pay')));
ok($paidD['ok'] === true && (int) $paidD['order_id'] === (int) $shared['order_id'], 'shared-identity focusa order completed');
$journeys['shared'][] = 'order_complete';
$licenseD = $svc->issueLicense(array_merge(['registration_uuid' => $regD], $correlation(41, 'license')));
ok($licenseD['ok'] === true && (int) $licenseD['edd_license_id'] === (int) $shared['edd_license_id'], 'shared-identity focusa key issued');
ok($licenseD['grants'] === ['focusa_operator_lifetime_v1'], 'shared-identity key grants focusa only');
$maskD = $licenseD['license_key_mask'];
$journeys['shared'][] = $licenseD['state'];
$deliveryD = $svc->deliver(array_merge(['registration_uuid' => $regD], $correlation(42, 'deliver')));
ok($deliveryD['key_mask'] === $maskD, 'shared-identity dual-channel delivery');
$journeys['shared'][] = $deliveryD['state'];
$nodeD = $svc->registerNode(array_merge(['registration_uuid' => $regD, 'node_id' => $shared['node_id'], 'device_public_key' => $shared['device_public_key_b64']], $correlation(42, 'node')));
ok($nodeD['ok'] === true, 'shared-identity node registered');
$journeys['shared'][] = $nodeD['state'];
$leaseD = $svc->issueLease(array_merge(['registration_uuid' => $regD], $correlation(42, 'lease')));
ok($leaseD['ok'] === true && (int) $leaseD['sequence'] === 1, 'shared-identity focusa lease sequence 1 (independent per-product ledger)');
$journeys['shared'][] = $leaseD['state'];
$leaseRowD = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseD['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
$refundD = $svc->refund(array_merge(['registration_uuid' => $regD, 'reason' => 'synthetic_proof_cleanup'], $correlation(43, 'refund')));
ok($refundD['ok'] === true && (int) $refundD['sequence_after'] === 2, 'shared-identity refund increments the focusa ledger to 2');
$journeys['shared'][] = $refundD['state'];
$receiptD = $svc->receipt(['registration_uuid' => $regD]);
ok($receiptD['state'] === 'refunded' && preg_match('/^[0-9a-f]{64}$/', (string) $receiptD['receipt_sha256']) === 1, 'shared-identity receipt redacted with immutable handle');
// No duplicate identity/account exists for the shared email identity.
$identityRows = $db->query("SELECT COUNT(*) FROM wp_wpuiai_ubi_identities WHERE email_digest = '" . $bundle['email_digest'] . "'")->fetchColumn();
ok((int) $identityRows === 1, 'shared email identity has exactly ONE identity row (no duplicate customer identity)');
$sharedAccountRows = $db->query("SELECT COUNT(*) FROM wp_wpuiai_ubi_accounts WHERE account_uuid = '{$accountC}'")->fetchColumn();
ok((int) $sharedAccountRows === 1, 'shared email identity has exactly ONE account row');

// ── Session E: partial delivery (email bounce) then recovery ───────────────
$startE = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $partial['product_code'],
    'email_digest' => $partial['email_digest'], 'email_domain' => $partial['email_domain'],
    'email_prefix_char' => $partial['email_prefix_char'], 'challenge_code' => $partial['verification_code'],
], $correlation(50, 'start')));
ok($startE['ok'] === true, 'partial-delivery registration created');
$regE = $startE['registration_uuid'];
$journeys['partial'][] = $startE['state'];
$verifiedE = $svc->verifyEmail(array_merge(['registration_uuid' => $regE, 'code' => $partial['verification_code']], $correlation(50, 'verify')));
ok($verifiedE['ok'] === true, 'partial-delivery mailbox verified');
$journeys['partial'][] = $verifiedE['state'];
$promotedE = $svc->promote(array_merge(['registration_uuid' => $regE], $correlation(50, 'promote')));
ok((int) $promotedE['customer_id'] === (int) $partial['customer_id'], 'partial-delivery customer 2298 created');
$journeys['partial'][] = $promotedE['state'];
$intentE = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regE], $correlation(51, 'intent')));
ok($intentE['ok'] === true, 'partial-delivery checkout intent created');
$journeys['partial'][] = $intentE['state'];
$paidE = $svc->completePayment(array_merge([
    'registration_uuid' => $regE, 'checkout_email_digest' => $partial['email_digest'],
    'payment_reference_digest' => $partial['payment_reference_digest'],
], $correlation(51, 'pay')));
ok($paidE['ok'] === true && (int) $paidE['order_id'] === (int) $partial['order_id'], 'partial-delivery order completed');
$journeys['partial'][] = 'order_complete';
$licenseE = $svc->issueLicense(array_merge(['registration_uuid' => $regE], $correlation(51, 'license')));
ok($licenseE['ok'] === true && (int) $licenseE['edd_license_id'] === (int) $partial['edd_license_id'], 'partial-delivery key issued');
$maskE = $licenseE['license_key_mask'];
$journeys['partial'][] = $licenseE['state'];
okThrows(static fn() => $svc->recoverDelivery(array_merge(['registration_uuid' => $regE], $correlation(51, 'recover'))), 'PARTIAL_DELIVERY_REQUIRED', 'recovery before a partial delivery is denied');
// Email channel bounces (test-mode seam): account channel still delivers, but
// the license is NEVER silently granted and downstream steps fail closed.
$partialDelivery = $svc->deliver(array_merge(['registration_uuid' => $regE, 'email_bounce_seam' => true], $correlation(52, 'deliver')));
ok($partialDelivery['ok'] === true && $partialDelivery['state'] === 'delivery_partial', 'email bounce marks delivery partial (never silent success)');
ok($partialDelivery['channels'] === ['email' => 'failed', 'account' => 'sent'], 'partial delivery: email failed, account sent');
ok($partialDelivery['partial'] === true && $partialDelivery['recovery_required'] === true, 'partial delivery requires recovery');
ok($partialDelivery['key_mask'] === $maskE && $partialDelivery['same_canonical_key_both_channels'] === true, 'partial delivery still targets the one canonical key');
$journeys['partial'][] = 'delivery_partial';
okThrows(static fn() => $svc->registerNode(array_merge(['registration_uuid' => $regE, 'node_id' => $partial['node_id'], 'device_public_key' => $partial['device_public_key_b64']], $correlation(53, 'node'))), 'LICENSE_DELIVERY_PENDING', 'no node registration while delivery is partial');
okThrows(static fn() => $svc->issueLease(array_merge(['registration_uuid' => $regE], $correlation(53, 'lease'))), 'NODE_REQUIRED', 'no lease while delivery is partial');
okThrows(static fn() => $svc->deliver(array_merge(['registration_uuid' => $regE], $correlation(53, 'deliver'))), 'DELIVERY_ALREADY_PARTIAL', 'a second deliver attempt while partial is denied');
$recoveredE = $svc->recoverDelivery(array_merge(['registration_uuid' => $regE], $correlation(54, 'recover')));
ok($recoveredE['ok'] === true && $recoveredE['state'] === 'delivered', 'partial delivery recovered to delivered');
ok($recoveredE['channels'] === ['email' => 'sent', 'account' => 'sent'] && $recoveredE['same_canonical_key_both_channels'] === true, 'recovery completes both channels with the same canonical key');
ok($recoveredE['duplicate_key'] === false && $recoveredE['healthy_channel_not_redelivered'] === true, 'recovery never duplicates the key and never re-delivers the healthy channel');
$journeys['partial'][] = 'delivered';
$deliveryRowsE = $db->query("SELECT COUNT(*) FROM wp_wpuiai_ubi_deliveries WHERE edd_license_id = " . (int) $partial['edd_license_id'])->fetchColumn();
ok((int) $deliveryRowsE === 2, 'recovery keeps exactly two delivery rows (no duplicate delivery)');
$nodeE = $svc->registerNode(array_merge(['registration_uuid' => $regE, 'node_id' => $partial['node_id'], 'device_public_key' => $partial['device_public_key_b64']], $correlation(54, 'node')));
ok($nodeE['ok'] === true, 'node registration proceeds only after full delivery');
$journeys['partial'][] = $nodeE['state'];
$leaseE = $svc->issueLease(array_merge(['registration_uuid' => $regE], $correlation(54, 'lease')));
ok($leaseE['ok'] === true && (int) $leaseE['sequence'] === 1, 'lease issued only after full delivery, sequence 1');
$journeys['partial'][] = $leaseE['state'];
$leaseRowE = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseE['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
$refundE = $svc->refund(array_merge(['registration_uuid' => $regE, 'reason' => 'synthetic_proof_cleanup'], $correlation(55, 'refund')));
ok($refundE['ok'] === true && (int) $refundE['sequence_after'] === 2, 'partial-delivery session refunds to sequence 2');
$journeys['partial'][] = $refundE['state'];
$receiptE = $svc->receipt(['registration_uuid' => $regE]);
ok($receiptE['state'] === 'refunded' && preg_match('/^[0-9a-f]{64}$/', (string) $receiptE['receipt_sha256']) === 1, 'partial-delivery receipt redacted with immutable handle');

// ── Session G: refunded records never reactivate; a NEW purchase does ──────
// Purchase 1: verified purchase, lease, refund.
$startG1 = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $react['product_code'],
    'email_digest' => $react['email_digest'], 'email_domain' => $react['email_domain'],
    'email_prefix_char' => $react['email_prefix_char'], 'challenge_code' => $react['verification_code'],
], $correlation(60, 'start')));
ok($startG1['ok'] === true, 'reactivation purchase-1 registration created');
$regG1 = $startG1['registration_uuid'];
$journeys['reactivation_first'][] = $startG1['state'];
$verifiedG1 = $svc->verifyEmail(array_merge(['registration_uuid' => $regG1, 'code' => $react['verification_code']], $correlation(60, 'verify')));
ok($verifiedG1['ok'] === true, 'reactivation purchase-1 mailbox verified');
$journeys['reactivation_first'][] = $verifiedG1['state'];
$promotedG1 = $svc->promote(array_merge(['registration_uuid' => $regG1], $correlation(60, 'promote')));
ok((int) $promotedG1['customer_id'] === (int) $react['customer_id'], 'reactivation customer 2171 created');
$accountG = $promotedG1['account_uuid'];
$journeys['reactivation_first'][] = $promotedG1['state'];
$intentG1 = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regG1], $correlation(61, 'intent')));
ok($intentG1['ok'] === true, 'reactivation purchase-1 checkout intent created');
$journeys['reactivation_first'][] = $intentG1['state'];
$paidG1 = $svc->completePayment(array_merge([
    'registration_uuid' => $regG1, 'checkout_email_digest' => $react['email_digest'],
    'payment_reference_digest' => $react['payment_reference_digest_first'],
], $correlation(61, 'pay')));
ok($paidG1['ok'] === true && (int) $paidG1['order_id'] === (int) $react['first']['order_id'], 'reactivation purchase-1 order completed');
$journeys['reactivation_first'][] = 'order_complete';
$licenseG1 = $svc->issueLicense(array_merge(['registration_uuid' => $regG1], $correlation(61, 'license')));
ok($licenseG1['ok'] === true && (int) $licenseG1['edd_license_id'] === (int) $react['first']['edd_license_id'], 'reactivation purchase-1 key issued');
$maskG1 = $licenseG1['license_key_mask'];
$journeys['reactivation_first'][] = $licenseG1['state'];
$deliveryG1 = $svc->deliver(array_merge(['registration_uuid' => $regG1], $correlation(62, 'deliver')));
ok($deliveryG1['state'] === 'delivered', 'reactivation purchase-1 delivered');
$journeys['reactivation_first'][] = $deliveryG1['state'];
$nodeG1 = $svc->registerNode(array_merge(['registration_uuid' => $regG1, 'node_id' => $react['node_id'], 'device_public_key' => $react['device_public_key_b64']], $correlation(62, 'node')));
ok($nodeG1['ok'] === true, 'reactivation purchase-1 node registered');
$journeys['reactivation_first'][] = $nodeG1['state'];
$leaseG1 = $svc->issueLease(array_merge(['registration_uuid' => $regG1], $correlation(62, 'lease')));
ok($leaseG1['ok'] === true && (int) $leaseG1['sequence'] === 1, 'reactivation purchase-1 lease sequence 1');
$journeys['reactivation_first'][] = $leaseG1['state'];
$leaseRowG1 = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseG1['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
$refundG1 = $svc->refund(array_merge(['registration_uuid' => $regG1, 'reason' => 'synthetic_proof_cleanup'], $correlation(63, 'refund')));
ok($refundG1['ok'] === true && (int) $refundG1['sequence_after'] === 2, 'reactivation purchase-1 refund increments sequence to 2');
$journeys['reactivation_first'][] = $refundG1['state'];
$reactG1 = $svc->reactivate(array_merge(['registration_uuid' => $regG1], $correlation(64, 'reactivate')));
ok($reactG1['ok'] === false && $reactG1['error'] === 'REFUNDED_NEVER_REACTIVATES', 'refunded purchase never reactivates on its own');
ok($reactG1['reactivation_requires_new_purchase'] === true && $reactG1['refresh_denied'] === true, 'refunded reactivation requires a new verified purchase');
$orderStateG1 = $db->query("SELECT state FROM wp_wpuiai_ubi_orders WHERE order_id = " . (int) $react['first']['order_id'])->fetchColumn();
$licenseStateG1 = $db->query("SELECT state FROM wp_wpuiai_ubi_licenses WHERE edd_license_id = " . (int) $react['first']['edd_license_id'])->fetchColumn();
ok($orderStateG1 === 'refunded' && $licenseStateG1 === 'refunded', 'reactivation purchase-1 order/license rows stay refunded and preserved');
// Purchase 2: a NEW EDD order for the same verified identity — new key/lease at
// monotonic sequence 3; the refunded rows are never touched.
$startG2 = $svc->startRegistration(array_merge([
    'facade_id' => $facade['facade_id'], 'origin' => $facade['origin'],
    'product_code' => $react['product_code'],
    'email_digest' => $react['email_digest'], 'email_domain' => $react['email_domain'],
    'email_prefix_char' => $react['email_prefix_char'], 'challenge_code' => $react['verification_code'],
], $correlation(65, 'start')));
ok($startG2['ok'] === true, 'reactivation purchase-2 registration created (same verified identity)');
$regG2 = $startG2['registration_uuid'];
$journeys['reactivation_second'][] = $startG2['state'];
$verifiedG2 = $svc->verifyEmail(array_merge(['registration_uuid' => $regG2, 'code' => $react['verification_code']], $correlation(65, 'verify')));
ok($verifiedG2['ok'] === true, 'reactivation purchase-2 mailbox verified');
$journeys['reactivation_second'][] = $verifiedG2['state'];
$promotedG2 = $svc->promote(array_merge(['registration_uuid' => $regG2], $correlation(65, 'promote')));
ok($promotedG2['ok'] === true && (int) $promotedG2['customer_id'] === (int) $react['customer_id'] && $promotedG2['account_uuid'] === $accountG, 'purchase-2 reuses the same verified account');
ok($promotedG2['identity_reused'] === true && $promotedG2['zero_new_rows'] === true, 'purchase-2 promotion reuses identity with zero new rows');
$journeys['reactivation_second'][] = $promotedG2['state'];
$intentG2 = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regG2], $correlation(66, 'intent')));
ok($intentG2['ok'] === true, 'reactivation purchase-2 checkout intent created');
$journeys['reactivation_second'][] = $intentG2['state'];
$paidG2 = $svc->completePayment(array_merge([
    'registration_uuid' => $regG2, 'checkout_email_digest' => $react['email_digest'],
    'payment_reference_digest' => $react['payment_reference_digest_second'],
], $correlation(66, 'pay')));
ok($paidG2['ok'] === true && (int) $paidG2['order_id'] === (int) $react['second']['order_id'], 'reactivation purchase-2 is a NEW EDD order');
$journeys['reactivation_second'][] = 'order_complete';
$licenseG2 = $svc->issueLicense(array_merge(['registration_uuid' => $regG2], $correlation(66, 'license')));
ok($licenseG2['ok'] === true && (int) $licenseG2['edd_license_id'] === (int) $react['second']['edd_license_id'], 'reactivation purchase-2 issues a NEW canonical key');
$maskG2 = $licenseG2['license_key_mask'];
$journeys['reactivation_second'][] = $licenseG2['state'];
$deliveryG2 = $svc->deliver(array_merge(['registration_uuid' => $regG2], $correlation(67, 'deliver')));
ok($deliveryG2['key_mask'] === $maskG2, 'reactivation purchase-2 delivered with its own canonical key');
$journeys['reactivation_second'][] = $deliveryG2['state'];
$nodeG2 = $svc->registerNode(array_merge(['registration_uuid' => $regG2, 'node_id' => $react['node_id'], 'device_public_key' => $react['device_public_key_b64']], $correlation(67, 'node')));
ok($nodeG2['ok'] === true, 'reactivation purchase-2 node registered');
$journeys['reactivation_second'][] = $nodeG2['state'];
$leaseG2 = $svc->issueLease(array_merge(['registration_uuid' => $regG2], $correlation(67, 'lease')));
ok($leaseG2['ok'] === true && (int) $leaseG2['sequence'] === (int) $leaseCfg['sequence_after_reactivation'], 'reactivation purchase-2 lease at monotonic sequence 3');
$journeys['reactivation_second'][] = $leaseG2['state'];
$leaseRowG2 = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseG2['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
$receiptG = $svc->receipt(['registration_uuid' => $regG2]);
ok($receiptG['state'] === 'lease_issued' && (int) $receiptG['lease_sequence'] === 3 && $receiptG['lease_state'] === 'active', 'reactivation receipt shows the new active lease at sequence 3');
ok(preg_match('/^[0-9a-f]{64}$/', (string) $receiptG['receipt_sha256']) === 1, 'reactivation receipt immutable handle');
// Refunded truth preserved: the purchase-1 rows are untouched by purchase-2.
$orderStateG2 = $db->query("SELECT state FROM wp_wpuiai_ubi_orders WHERE order_id = " . (int) $react['second']['order_id'])->fetchColumn();
$licenseStateG2 = $db->query("SELECT state FROM wp_wpuiai_ubi_licenses WHERE edd_license_id = " . (int) $react['second']['edd_license_id'])->fetchColumn();
$leaseStateG1 = $db->query("SELECT state FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseG1['lease_uuid'] . "'")->fetchColumn();
$leaseStateG2 = $db->query("SELECT state FROM wp_wpuiai_ubi_leases WHERE lease_uuid = '" . $leaseG2['lease_uuid'] . "'")->fetchColumn();
ok($orderStateG2 === 'complete' && $licenseStateG2 === 'active', 'purchase-2 order/license are active');
ok($leaseStateG1 === 'refunded' && $leaseStateG2 === 'active', 'refunded lease stays refunded; new lease is active');

// ── Final counts, sequences, receipts, and preservation ────────────────────
$final = $counts();
$vectors = $fixture['journey']['journal_vectors'];
ok($final['registrations'] === $vectors['registrations'], 'registration row count pinned');
ok($final['identities'] === $vectors['identities'] && $final['accounts'] === $vectors['accounts'], 'identity/account row counts pinned');
ok($final['orders'] === $vectors['orders'] && $final['order_items'] === $vectors['order_items'], 'order/order-item row counts pinned');
ok($final['licenses'] === $vectors['licenses'] && $final['deliveries'] === $vectors['deliveries'], 'license/delivery row counts pinned');
ok($final['nodes'] === $vectors['nodes'] && $final['leases'] === $vectors['leases'], 'node/lease row counts pinned');
ok($final['sequences'] === $vectors['sequences'] && $final['refunds'] === $vectors['refunds'], 'sequence/refund row counts pinned');
ok($final['child_tokens'] === $vectors['child_tokens'], 'child-token row count pinned');
ok($final['journal'] === $vectors['journal_events'], 'journal event count pinned');
$seqOf = static function (string $registrationUuid) use ($db): int {
    return (int) $db->query("SELECT current_sequence FROM wp_wpuiai_ubi_sequences WHERE account_uuid = (SELECT account_uuid FROM wp_wpuiai_ubi_registrations WHERE registration_uuid = '{$registrationUuid}') AND product_code = (SELECT product_code FROM wp_wpuiai_ubi_registrations WHERE registration_uuid = '{$registrationUuid}')")->fetchColumn();
};
ok($seqOf($regA) === 2 && $seqOf($regB) === 2 && $seqOf($regC) === 2, 'focusa/uiai/bundle sequence ledgers end at 2 after refund');
ok($seqOf($regD) === 2 && $seqOf($regE) === 2, 'shared/partial sequence ledgers end at 2 after refund');
ok($seqOf($regG2) === 3, 'reactivation sequence ledger ends at 3 after the new purchase');
$preserved = $schema->preserveForRollback('2026-08-09T08:00:00Z', ['source' => 'uiai_bundle_isolation_rollback']);
ok($preserved['action'] === 'preserve', 'rollback is preservation-only');

$leaseEnvelopes = [
    'focusa' => ['payload_b64' => $leaseRowA['payload_b64'], 'signature_b64' => $leaseRowA['signature_b64']],
    'uiai' => ['payload_b64' => $leaseRowB['payload_b64'], 'signature_b64' => $leaseRowB['signature_b64']],
    'bundle' => ['payload_b64' => $leaseRowC['payload_b64'], 'signature_b64' => $leaseRowC['signature_b64']],
    'shared' => ['payload_b64' => $leaseRowD['payload_b64'], 'signature_b64' => $leaseRowD['signature_b64']],
    'partial' => ['payload_b64' => $leaseRowE['payload_b64'], 'signature_b64' => $leaseRowE['signature_b64']],
    'reactivation_first' => ['payload_b64' => $leaseRowG1['payload_b64'], 'signature_b64' => $leaseRowG1['signature_b64']],
    'reactivation_second' => ['payload_b64' => $leaseRowG2['payload_b64'], 'signature_b64' => $leaseRowG2['signature_b64']],
];
$summary = [
    'schema' => 'focusa.spec152e.uiai_bundle_isolation_test.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'journeys' => $journeys,
    'counts' => $final,
    'sequences' => [
        'focusa' => $seqOf($regA), 'uiai' => $seqOf($regB), 'bundle' => $seqOf($regC),
        'shared' => $seqOf($regD), 'partial' => $seqOf($regE), 'reactivation' => $seqOf($regG2),
    ],
    'shared_identity' => [
        'bundle_account' => $accountC, 'shared_account' => $promotedD['account_uuid'],
        'bundle_customer' => (int) $bundle['customer_id'], 'shared_customer' => (int) $promotedD['customer_id'],
        'account_reused' => $promotedD['account_uuid'] === $accountC,
    ],
    'key_masks' => [
        'focusa' => $maskA, 'uiai' => $maskB, 'bundle' => $maskC,
        'shared' => $maskD, 'partial' => $maskE,
        'reactivation_first' => $maskG1, 'reactivation_second' => $maskG2,
    ],
    'lease_envelopes' => $leaseEnvelopes,
    'child_tokens' => [
        'uiai' => $ctB['child_token'] + ['token_digest' => $ctB['token_digest'], 'token_stored' => $ctB['token_stored'], 'max_ttl_minutes' => (int) $ctB['max_ttl_minutes']],
        'bundle' => $ctC['child_token'] + ['token_digest' => $ctC['token_digest'], 'token_stored' => $ctC['token_stored'], 'max_ttl_minutes' => (int) $ctC['max_ttl_minutes']],
    ],
    'partial_delivery' => [
        'email_failed' => $partialDelivery['channels']['email'] === 'failed',
        'account_sent' => $partialDelivery['channels']['account'] === 'sent',
        'blocked_node_and_lease' => true,
        'recovered' => $recoveredE['ok'] === true,
        'delivery_rows' => (int) $deliveryRowsE,
    ],
    'reactivation' => [
        'denied' => true, 'code' => 'REFUNDED_NEVER_REACTIVATES',
        'new_lease_sequence' => (int) $leaseG2['sequence'],
    ],
    'wrong_product' => ['unknown_code_denied' => true, 'no_cross_product_lease' => true],
    'receipt_sha256' => [
        'focusa' => $receiptA['receipt_sha256'], 'uiai' => $receiptB['receipt_sha256'],
        'bundle' => $receiptC['receipt_sha256'], 'shared' => $receiptD['receipt_sha256'],
        'partial' => $receiptE['receipt_sha256'], 'reactivation' => $receiptG['receipt_sha256'],
    ],
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
"""


def run_harness() -> str:
    if not PHP:
        raise AssertionError("FAIL: php is required to execute the UIAI/bundle isolation journey")
    with tempfile.TemporaryDirectory() as tmp:
        harness_path = Path(tmp) / "uiai_bundle_harness.php"
        harness_path.write_text(HARNESS, encoding="utf-8")
        proc = subprocess.run(
            [PHP, str(harness_path), str(LEASE_CONTRACT), str(PROJECTOR_CONTRACT), str(EXCLUSION_CONTRACT), str(CONTRACT), str(FIXTURE)],
            capture_output=True, text=True, timeout=240,
        )
        if proc.returncode != 0:
            raise AssertionError(f"FAIL: php harness exited {proc.returncode}: {proc.stderr[:3000]}")
        return proc.stdout.strip()


first = run_harness()
second = run_harness()
expect(first == second, "harness output is byte-identical across runs (replayable)")
result = json.loads(first)
expect(result["result"] == "passed_fail_closed", "harness passed fail-closed")

fixture_raw = FIXTURE.read_text(encoding="utf-8")
fixture = json.loads(fixture_raw)
contract_raw = CONTRACT.read_text(encoding="utf-8")
lease_contract_raw = LEASE_CONTRACT.read_text(encoding="utf-8")
exclusion_contract_raw = EXCLUSION_CONTRACT.read_text(encoding="utf-8")

# ── Signed leases: independent Ed25519 verification (byte-compatible) ───────

lease_domain = fixture["lease"]["domain"].encode()
lease_public_key = base64.b64decode(fixture["lease"]["lease_public_key_b64"])
sessions = fixture["sessions"]
expected_lease_grants = {
    "focusa": ["focusa_operator_lifetime_v1"],
    "uiai": ["uiai_operator_lifetime_v1"],
    "bundle": ["focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1"],
    "shared": ["focusa_operator_lifetime_v1"],
    "partial": ["focusa_operator_lifetime_v1"],
    "reactivation_first": ["focusa_operator_lifetime_v1"],
    "reactivation_second": ["focusa_operator_lifetime_v1"],
}
expected_products = {
    "focusa": ["focusa"],
    "uiai": ["uiai_engine"],
    "bundle": ["focusa", "uiai_engine"],
    "shared": ["focusa"],
    "partial": ["focusa"],
    "reactivation_first": ["focusa"],
    "reactivation_second": ["focusa"],
}
expected_sequences = {
    "focusa": 1, "uiai": 1, "bundle": 1, "shared": 1, "partial": 1,
    "reactivation_first": 1, "reactivation_second": 3,
}
expected_edd = {
    "focusa": (sessions["focusa"]["customer_id"], sessions["focusa"]),
    "uiai": (sessions["uiai"]["customer_id"], sessions["uiai"]),
    "bundle": (sessions["bundle"]["customer_id"], sessions["bundle"]),
    "shared": (sessions["shared"]["customer_id"], sessions["shared"]),
    "partial": (sessions["partial"]["customer_id"], sessions["partial"]),
    "reactivation_first": (sessions["reactivation"]["customer_id"], sessions["reactivation"]["first"]),
    "reactivation_second": (sessions["reactivation"]["customer_id"], sessions["reactivation"]["second"]),
}
for key in expected_lease_grants:
    lease_env = result["lease_envelopes"][key]
    payload_bytes = base64.b64decode(lease_env["payload_b64"])
    signature = base64.b64decode(lease_env["signature_b64"])
    public_key = Ed25519PublicKey.from_public_bytes(lease_public_key)
    try:
        public_key.verify(signature, lease_domain + payload_bytes)
        expect(True, f"lease signature verifies (Ed25519, domain-separated) for {key}")
    except InvalidSignature as exc:  # pragma: no cover - only on signature breakage
        raise AssertionError(f"FAIL: lease signature did not verify for {key}: {exc}")
    lease_payload = json.loads(payload_bytes)
    expect(lease_payload["schema"] == "focusa.authority_lease.v1", f"lease payload schema for {key}")
    expect(lease_payload["sequence"] == expected_sequences[key] and lease_payload["status"] == "active", f"lease sequence {expected_sequences[key]} active for {key}")
    expect(lease_payload["grants"] == expected_lease_grants[key], f"lease grants exactly {expected_lease_grants[key]} for {key}")
    expect(lease_payload["products"] == expected_products[key], f"lease products exactly {expected_products[key]} for {key}")
    expect(lease_payload["install_site_authority"] == "none" and lease_payload["spec158"] == "excluded", f"lease posture for {key}")
    customer_id, edd = expected_edd[key]
    expect(lease_payload["customer_id"] == customer_id and lease_payload["order_id"] == edd["order_id"] and lease_payload["edd_license_id"] == edd["edd_license_id"], f"lease binds customer/order/license truth for {key}")
    expect(lease_payload["subject_id"] == result["shared_identity"]["bundle_account"] if key in ("bundle", "shared") else True, f"lease subject binds the account for {key}")
    expect(re.fullmatch(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", lease_payload["node_id"]) is not None or re.fullmatch(r"node-[a-z0-9-]+", lease_payload["node_id"]) is not None, f"lease binds the registered node for {key}")
    if key == "bundle":
        expect(lease_payload["posture"] == "bundle" and lease_payload["grant_composition"] == "exact_union", "bundle lease posture and exact-union composition")
        expect(lease_payload["human_key_count"] == 1, "bundle lease carries one human key")
        focusa_features = set(fixture["products"]["focusa_operator_lifetime_v1"]["features"])
        uiai_features = set(fixture["products"]["uiai_operator_lifetime_v1"]["features"])
        expect(set(lease_payload["features"]) == focusa_features | uiai_features, "bundle lease features are the derived union of both products")
        expect(lease_payload["limits"] == fixture["products"]["focusa_uiai_operator_bundle_lifetime_v1"]["limits"], "bundle lease limits frozen")

# ── Expected canonical keys match every delivered key mask ─────────────────

for key in expected_edd:
    _, edd = expected_edd[key]
    expected_key = derive_expected_key(edd["edd_license_id"], edd["order_id"])
    mask = result["key_masks"][key]
    expect(mask == expected_key[:11] + "-****-****-****", f"delivered key mask matches the canonical EDD key for {key}")

# ── Child tokens: bounded TTL, exact subset, digest-only ───────────────────

for key in ("uiai", "bundle"):
    token = result["child_tokens"][key]
    expect(token["schema"] == "focusa.uiai_child_token.v1", f"child token schema for {key}")
    expect(token["audience"] == fixture["child_token"]["audience"], f"child token audience for {key}")
    expect(token["features"] == fixture["child_token"]["features"], f"child token features are exactly the UIAI local families for {key}")
    expect(token["limits"] == fixture["child_token"]["limits"], f"child token limits frozen for {key}")
    issued = re.match(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})Z$", token["issued_at"])
    expires = re.match(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})Z$", token["expires_at"])
    expect(issued is not None and expires is not None, f"child token timestamps parse for {key}")
    issued_dt = _dt.datetime.fromisoformat(issued.group(1))
    expires_dt = _dt.datetime.fromisoformat(expires.group(1))
    ttl_minutes = (expires_dt - issued_dt).total_seconds() / 60
    expect(0 < ttl_minutes <= fixture["child_token"]["max_ttl_minutes"], f"child token TTL bounded to 15 minutes for {key}")
    expect(token["max_ttl_minutes"] == fixture["child_token"]["max_ttl_minutes"], f"child token max TTL pinned for {key}")
    expect(re.fullmatch(r"[0-9a-f]{64}", token["token_digest"]) is not None, f"child token digest 64-hex for {key}")
    expect(token["token_stored"] == "digest_only", f"child token digest-only at rest for {key}")
    expect("token" not in token and "token_secret" not in token, f"child token secret never leaves the authority for {key}")

# ── Shared identity ─────────────────────────────────────────────────────────

expect(result["shared_identity"]["account_reused"] is True, "shared identity reuses the bundle account")
expect(result["shared_identity"]["bundle_account"] == result["shared_identity"]["shared_account"], "shared and bundle registrations share one account uuid")
expect(result["shared_identity"]["bundle_customer"] == result["shared_identity"]["shared_customer"] == 2442, "shared and bundle registrations share one EDD customer id")

# ── Partial delivery and reactivation facts ────────────────────────────────

expect(result["partial_delivery"]["email_failed"] is True and result["partial_delivery"]["account_sent"] is True, "partial delivery: email failed, account sent (no silent success)")
expect(result["partial_delivery"]["blocked_node_and_lease"] is True and result["partial_delivery"]["recovered"] is True, "partial delivery blocks node/lease until recovery")
expect(result["partial_delivery"]["delivery_rows"] == 2, "recovery keeps exactly two delivery rows")
expect(result["reactivation"]["denied"] is True and result["reactivation"]["code"] == "REFUNDED_NEVER_REACTIVATES", "refunded record never reactivates")
expect(result["reactivation"]["new_lease_sequence"] == 3, "new purchase reactivates at monotonic sequence 3")
expect(result["wrong_product"]["unknown_code_denied"] is True and result["wrong_product"]["no_cross_product_lease"] is True, "wrong product fails closed")

# ── Fixture structure and expectations ─────────────────────────────────────

expect(fixture["schema"] == "focusa.spec152e.uiai_bundle_isolation_fixture.v1", "fixture schema id")
expect(fixture["fixture_id"] == "focusa-vbcqu.20.13.59", "fixture_id")
expect(fixture["fixture_kind"] == "public_synthetic_nonproduction", "public synthetic non-production fixture")
expect(fixture["authority"]["canonical"] == "WPUIAI.com EDD", "canonical authority")
expect(fixture["authority"]["new_issuance"] == "edd_authority_only", "new issuance edd only")
expect(fixture["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facade presenter/proxy only")
expect(fixture["authority"]["install_site_authority"] == "none", "no install-site authority")
expect(fixture["authority"]["spec158"] == "excluded", "spec158 excluded")
expect(fixture["redaction"] == {
    "raw_email": "absent", "raw_key": "absent", "payment_id_stored": False,
    "secret_material": "absent", "child_token": "digest_only_at_rest",
    "receipt": "masked_email_and_key_mask_only",
}, "redaction posture")
expect(fixture["products"]["focusa_operator_lifetime_v1"]["grants"] == ["focusa_operator_lifetime_v1"], "fixture focusa grants exact")
expect(fixture["products"]["uiai_operator_lifetime_v1"]["grants"] == ["uiai_operator_lifetime_v1"], "fixture uiai grants exact")
expect(fixture["products"]["focusa_uiai_operator_bundle_lifetime_v1"]["grant_composition"] == "exact_union", "fixture bundle composition exact union")
expect(fixture["products"]["focusa_uiai_operator_bundle_lifetime_v1"]["price_usd"] == "1254.60", "fixture bundle price canonical")
expect(fixture["lease"]["domain"] == "FOCUSA-AUTHORITY-LEASE-V1\0", "lease domain separation")
expect(fixture["child_token"]["max_ttl_minutes"] == 15, "child token TTL bound")
for key, value in fixture["journey"]["expectations"].items():
    expect(value is True, f"fixture expectation {key} pinned")
expect(fixture["journey"]["journal_vectors"]["accounts"] == 5 and fixture["journey"]["journal_vectors"]["identities"] == 5, "five unique verified identities/accounts")

# ── Contract static invariants ─────────────────────────────────────────────

expect("final class FocusaSpec152eUiaiBundleIsolationMigration" in contract_raw, "migration class")
expect("final class FocusaSpec152eUiaiBundleIsolationService" in contract_raw, "service class")
expect("focusa.spec152e.uiai_bundle_isolation.v1" in contract_raw, "contract schema id")
expect("focusa.spec152e.uiai_bundle_receipt.v1" in contract_raw, "receipt schema id")
for table in (
    "wpuiai_ubi_registrations", "wpuiai_ubi_identities", "wpuiai_ubi_accounts", "wpuiai_ubi_orders",
    "wpuiai_ubi_order_items", "wpuiai_ubi_licenses", "wpuiai_ubi_deliveries", "wpuiai_ubi_nodes",
    "wpuiai_ubi_leases", "wpuiai_ubi_sequences", "wpuiai_ubi_refunds", "wpuiai_ubi_child_tokens",
    "wpuiai_ubi_journal",
):
    expect(table in contract_raw, f"table {table}")
for method in (
    "function startRegistration", "function verifyEmail", "function promote",
    "function createCheckoutIntent", "function completePayment", "function issueLicense",
    "function deliver", "function recoverDelivery", "function registerNode",
    "function issueLease", "function issueChildToken", "function refund",
    "function reactivate", "function resolveProductGrants", "function receipt",
):
    expect(method in contract_raw, f"method {method}")
for code in (
    "FACADE_ORIGIN_DENIED", "PRODUCT_MAPPING_REQUIRED", "CALLER_CONTROLLED_GRANT_DENIED",
    "EMAIL_VERIFICATION_REQUIRED", "EMAIL_VERIFICATION_FAILED", "EMAIL_VERIFICATION_EXPIRED",
    "EDD_CHECKOUT_REQUIRED", "EDD_ORDER_UNVERIFIED", "EDD_ORDER_PENDING",
    "EDD_LICENSE_PENDING", "EDD_LICENSE_UNUSABLE", "NODE_NOT_FOUND",
    "NODE_REQUIRED", "LEASE_REQUIRED", "CHILD_TOKEN_NOT_INCLUDED",
    "REFUND_STATE_REQUIRED", "REFUND_REASON_REQUIRED", "REFUNDED_NEVER_REACTIVATES",
    "REACTIVATION_REQUIRES_NEW_ORDER", "REQUEST_ID_REQUIRED", "IDEMPOTENCY_KEY_REQUIRED",
    "IDEMPOTENCY_CONFLICT", "REGISTRATION_NOT_FOUND", "LICENSE_DELIVERY_PENDING",
    "PARTIAL_DELIVERY_PENDING", "PARTIAL_DELIVERY_REQUIRED", "DELIVERY_ALREADY_PARTIAL",
    "DELIVERY_ALREADY_DELIVERED", "DEVICE_PUBLIC_KEY_REQUIRED",
):
    expect(code in contract_raw, f"fail-closed code {code}")
for code in ("INVALID_BASE64", "INVALID_PAYLOAD", "INVALID_PUBLIC_KEY", "INVALID_SIGNATURE"):
    expect(code in lease_contract_raw, f"lease envelope fail-closed code {code}")
expect("UiaiSpec172HostedResourceExclusionRegistry" in contract_raw and "exclusionList" in exclusion_contract_raw, "frozen hosted-resource exclusion registry required")
expect("HOSTED_RESOURCE_NOT_INCLUDED" in exclusion_contract_raw, "hosted-resource exclusion code")
expect("FocusaSpec152eEd25519Signer::LEASE_DOMAIN" in contract_raw, "canonical lease-signing domain")
expect("focusa.uiai_child_token.v1" in contract_raw, "child-token schema matches the runtime broker")
expect("CHILD_TOKEN_MAX_TTL_MINUTES" in contract_raw and "15" in contract_raw, "child-token TTL bound")
expect("token_digest" in contract_raw and "digest_only" in contract_raw, "child token digest-only at rest")
expect("bundleFeaturesDerived" in contract_raw and "array_merge(self::FOCUSA_FEATURES, self::UIAI_FEATURES)" in contract_raw, "bundle features are the DERIVED union, never a third list")
expect("exact_union" in contract_raw and "human_key_count" in contract_raw, "bundle exact union and one human key")
expect("identity_key" in contract_raw and "identity_reused" in contract_raw, "shared identity keyed by verified email, never by product")
expect("REFUNDED_NEVER_REACTIVATES" in contract_raw, "refunded records never reactivate")
expect("hash_equals" in contract_raw, "constant-time digest comparison")
expect("spec158" not in contract_raw or "excluded" in contract_raw, "spec158 excluded asserted")
expect("install_site_authority" in contract_raw, "install-site authority posture asserted")
expect("INSERT OR IGNORE" not in contract_raw or "preserveForRollback" in contract_raw, "preservation seam present")
# Preservation-only: no destructive path may exist anywhere in the contract.
for forbidden in ("DELETE FROM", "TRUNCATE", "DROP TABLE", "DROP INDEX"):
    expect(forbidden not in contract_raw, f"no destructive statement {forbidden}")
# No raw email or client-controlled price/grant inputs in the contract.
expect("customer_email" not in contract_raw and "raw_email TEXT" not in contract_raw and "raw_email VARCHAR" not in contract_raw, "no raw email storage field")
expect("'price' =>" not in contract_raw and "['price']" not in contract_raw, "no client-controlled price input")
expect("$request['grant']" not in contract_raw and "$request['grants']" not in contract_raw, "no client-controlled grant input (server-owned grants only)")
expect("challenge_hash" in contract_raw and "challenge_used" in contract_raw, "challenge hashed, single-use at rest")

# ── Redaction: no secret or unmasked real-email evidence in artifacts ───────

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
SYNTHETIC_KEY_RE = re.compile(r"(?i)focusa_live_[0-9]+_[0-9a-f]+")
PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
GITHUB_TOKEN_RE = re.compile(r"ghp_[A-Za-z0-9]{8,}")
LICENSE_SHAPE_RE = re.compile(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}")
for name, raw in (("fixture", fixture_raw), ("contract", contract_raw), ("harness_output", first)):
    expect(EMAIL_RE.search(raw) is None, f"no email literal in {name}")
    expect(SECRET_RE.search(raw) is None, f"no stripe secret prefix in {name}")
    expect(SYNTHETIC_KEY_RE.search(raw) is None, f"no synthetic focusa_live key in {name}")
    expect(PRIVATE_KEY_RE.search(raw) is None, f"no private key material in {name}")
    expect(GITHUB_TOKEN_RE.search(raw) is None, f"no GitHub token in {name}")
    expect(LICENSE_SHAPE_RE.search(raw) is None, f"no raw license-shaped evidence in {name}")
    expect("payment_intent_id" not in raw, f"no payment intent id in {name}")
expect("FOCUSA-****-****-****-****" in fixture_raw, "fixture pins the masked key form")
expect("f***@invalid.example" not in fixture_raw and "f***[AT]invalid.example" not in fixture_raw, "fixture carries no email literal at all")

# ── Journey/journal vectors from the harness ────────────────────────────────

for key in ("focusa", "uiai", "bundle", "shared", "partial", "reactivation_first", "reactivation_second"):
    expect(result["journeys"][key] == fixture["journey"][f"states_{key}"], f"journey states match the pinned state machine for {key}")
expected_counts = dict(fixture["journey"]["journal_vectors"])
expected_counts["journal"] = expected_counts.pop("journal_events")
expect(result["counts"] == expected_counts, "row counts match the pinned journal vectors")
expect(result["sequences"] == {"focusa": 2, "uiai": 2, "bundle": 2, "shared": 2, "partial": 2, "reactivation": 3}, "sequence ledgers pinned")
for key in result["receipt_sha256"]:
    expect(re.fullmatch(r"[0-9a-f]{64}", str(result["receipt_sha256"][key])) is not None, f"immutable receipt handle {key} is 64-hex")

positive_checks = result["positive_checks"]
negative_checks = result["negative_checks"]
gate_static_checks = positive - positive_checks

summary = {
    "schema": "focusa.spec152e.uiai_bundle_isolation_e2e_validation.v1",
    "atom": "focusa-vbcqu.20.13.59",
    "fixture_sha256": sha256_text(fixture_raw),
    "contract_sha256": sha256_text(contract_raw),
    "harness_sha256": sha256_text(first),
    "positive_checks": positive_checks,
    "negative_checks": negative_checks,
    "gate_static_checks": gate_static_checks,
    "journeys": {key: len(result["journeys"][key]) for key in result["journeys"]},
    "sequences": result["sequences"],
    "shared_identity_account_reused": result["shared_identity"]["account_reused"],
    "bundle_one_key_one_account": True,
    "partial_delivery_never_silent": result["partial_delivery"]["email_failed"] and result["partial_delivery"]["account_sent"],
    "partial_delivery_recovered": result["partial_delivery"]["recovered"],
    "refunded_never_reactivates": result["reactivation"]["denied"],
    "reactivation_new_lease_sequence": result["reactivation"]["new_lease_sequence"],
    "lease_signature_verified": True,
    "child_token_ttl_bounded": True,
    "child_token_digest_only": True,
    "key_masks_match_canonical": True,
    "harness_replay_identical": True,
    "result": "passed",
}
print(json.dumps(summary, sort_keys=True))
