#!/usr/bin/env python3
"""Spec 152E.07.03 terminal + agent paid activation against test authority
(atom focusa-vbcqu.20.13.57).

Exact verification:
    python3 tests/spec152e_paid_terminal_agent_e2e_test.py

Runs two synthetic test-mode sessions that settle the same semantic paid flow
(spec 152E §14, §23 rows "Terminal paid Focusa" and "Agent paid Focusa") plus
one abandoned session for bounded-poll budget exhaustion, all on sqlite
through a deterministic PHP harness whose output is byte-identical across
runs (replayable from the pinned commit):

- Terminal session (presenter cli_terminal): verified email -> facade checkout
  link -> bounded poll -> EDD human key -> dual key delivery (transactional
  email + one-time device-encrypted terminal envelope) -> protected credential
  store -> explicit customer-controlled key reveal -> node registration ->
  signed device-bound lease -> EDD refund cleanup (sequence increment,
  recovery_only, truth preserved).
- Agent session (presenter agent_json): structured challenge -> human
  verification/payment -> pause/resume with the protected poll credential ->
  bounded poll within budget -> masked agent transcript (focusa.agent_
  activation_envelope.v1, never the envelope/credential/key) -> credential
  store + explicit reveal -> signed lease -> refund cleanup.
- Abandoned agent session: polls beyond the 40-poll budget cancel fail-closed
  to recovery_only with zero order/license/node/lease rows.

Python independently re-verifies every signed lease with `cryptography`
Ed25519PublicKey over the domain-separated payload (byte-compatible with the
Rust verifier), decrypts both one-time terminal envelopes with X25519 +
HKDF-SHA256 + AES-256-GCM and proves the revealed key is the same canonical
EDD key that was emailed (same canonical key both channels), and asserts the
masked agent transcript contains no forbidden field. Secrets and unmasked real
email are absent from every artifact; the immutable receipt sha256 handle is
recorded.

Surfaces under test (EXACT SURFACES):
- Interactive CLI and agent JSON: docs/contracts/spec152e-terminal-agent-paid-
  activation.v1.php (startRegistration, verifyEmail, promote, poll, pause,
  resume, agentStatus, openEnvelope, credentialStore, revealKey) + fixture
  docs/contracts/spec152e-terminal-agent-paid-activation-fixture.v1.json
- Verification: single-use attempt-bounded mailbox challenge
- Checkout link: server-owned facade checkout URL (origin + path + token)
- Bounded poll: poll_count/max_polls=40; exhaustion cancels to recovery_only
- Dual key delivery: transactional email + one-time device-encrypted envelope
  (canonical X25519/HKDF/AES-GCM from docs/contracts/spec152e-terminal-
  delivery-envelope.v1.php)
- Credential store: protected store handle, mask, digest only; one-time reveal
- Lease: signed device-bound lease via the canonical Ed25519 key-set seam from
  docs/contracts/spec152e-edd-bound-lease-issuer.v1.php (loaded first)

Fail-closed invariants (spec 152E FORBIDDEN + §19 + §14.2):
- No unverified-email promotion: a submitted email creates only a pending
  attempt; no customer/account/order/license/node/lease exists until mailbox
  control is verified.
- No local/self-issued entitlement: the human key is created only by the EDD
  Software Licensing issuance step after a complete, integrity-ok order.
- No independent facade authority: facades are allowlisted presenters;
  caller-supplied origins/redirects/price/grants fail closed.
- Checkout email integrity: a different checkout email holds fulfillment.
- Bounded poll and pause/resume: agent-only; terminal registrations refuse
  resume steps; terminal states refuse pause/resume; credentials expire.
- Explicit key reveal: masked by default; reveal requires opt-in AND
  confirmation, is one-time, stays within the envelope lifetime, and refuses
  after the registration settles.
- One-time envelope never enters an agent transcript.
- Spec 158 implementation excluded; no raw email, raw key, payment reference,
  or secret material in artifacts; receipts masked with immutable sha256.
- No push, deploy, release, merge, or Beads mutation is performed.
"""

import base64
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from cryptography.exceptions import InvalidSignature, InvalidTag
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey, X25519PublicKey
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.hkdf import HKDF

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec152e-terminal-agent-paid-activation.v1.php"
LEASE_CONTRACT = ROOT / "docs/contracts/spec152e-edd-bound-lease-issuer.v1.php"
ENVELOPE_CONTRACT = ROOT / "docs/contracts/spec152e-terminal-delivery-envelope.v1.php"
FIXTURE = ROOT / "docs/contracts/spec152e-terminal-agent-paid-activation-fixture.v1.json"

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


def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()


def b64url_decode(data: str) -> bytes:
    padding = "=" * ((4 - len(data) % 4) % 4)
    return base64.urlsafe_b64decode(data + padding)


def derive_expected_key(license_id: int, order_id: int) -> str:
    raw = hashlib.sha256(f"edd-sl-v1\n{license_id}\n{order_id}".encode()).hexdigest().upper()[:32]
    return "-".join(raw[i:i + 8] for i in range(0, 32, 8))


def open_terminal_envelope(device_private: bytes, envelope: dict, info: bytes) -> bytes:
    """Open the one-time device-encrypted envelope (X25519 + HKDF + AES-256-GCM),
    byte-compatible with the PHP contract and libsodium."""
    if envelope.get("schema") != "focusa.spec152e.terminal_delivery_envelope.v1":
        raise ValueError("schema mismatch")
    if envelope.get("version") != 1:
        raise ValueError("version mismatch")
    if envelope.get("algorithm") != "X25519+HKDF-SHA256+AES-256-GCM":
        raise ValueError("algorithm mismatch")
    eph_public = b64url_decode(envelope["ephemeral_public_key"])
    nonce = b64url_decode(envelope["nonce"])
    sealed = b64url_decode(envelope["ciphertext"])
    shared = X25519PrivateKey.from_private_bytes(device_private).exchange(
        X25519PublicKey.from_public_bytes(eph_public)
    )
    key = HKDF(algorithm=hashes.SHA256(), length=32, salt=None, info=info).derive(shared)
    header = {k: envelope[k] for k in ("schema", "version", "algorithm", "ephemeral_public_key", "nonce")}
    aad = json.dumps(header, sort_keys=True, separators=(",", ":")).encode()
    return AESGCM(key).decrypt(nonce, sealed, aad)


# ── Deterministic PHP journey harness ───────────────────────────────────────

HARNESS = r"""<?php
// Spec 152E.07.03 terminal + agent paid activation journey harness (generated
// by the python gate). Executes the complete acceptance-matrix rows on sqlite
// with a fixed clock and deterministic crypto test seams, then emits a
// deterministic redacted summary. Byte-identical across runs; every
// positive/negative check is counted. No raw email, raw key, payment
// reference, or secret material ever appears in the summary.
declare(strict_types=1);
$leaseContractPath = $argv[1];
$envelopeContractPath = $argv[2];
$contractPath = $argv[3];
$fixturePath = $argv[4];
require_once $leaseContractPath;
require_once $envelopeContractPath;
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
    $ts = (new DateTimeImmutable('2026-08-09T05:00:00Z'))->modify('+' . ($tick * 10) . ' seconds')->format('Y-m-d\TH:i:s\Z');
    $tick++;
    return $ts;
};
$schema = new FocusaSpec152eTerminalAgentPaidMigration($db, 'wp_');
$schema->migrate('2026-08-09T04:00:00Z', ['source' => 'terminal_agent_paid_test']);
$keySet = new FocusaSpec152eAuthorityKeySetSeam(str_repeat('R', 32), str_repeat('L', 32), $clock);
$svc = new FocusaSpec152eTerminalAgentPaidService($db, $clock, 'wp_', $keySet);

$counts = static function () use ($db): array {
    $table = static fn(string $name): int => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_ta_{$name}")->fetchColumn();
    return [
        'registrations' => $table('registrations'),
        'identities' => $table('identities'),
        'accounts' => $table('accounts'),
        'orders' => $table('orders'),
        'order_items' => $table('order_items'),
        'licenses' => $table('licenses'),
        'deliveries' => $table('deliveries'),
        'envelopes' => $table('envelopes'),
        'credential_stores' => $table('credential_stores'),
        'nodes' => $table('nodes'),
        'leases' => $table('leases'),
        'sequences' => $table('sequences'),
        'refunds' => $table('refunds'),
        'journal' => $table('journal'),
    ];
};
$correlation = static function (int $seq, string $kind): array {
    return [
        'request_id' => 'req_ta_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
        'idempotency_key' => 'idem_ta_' . $kind . '_' . str_pad((string) $seq, 4, '0', STR_PAD_LEFT),
    ];
};
$journeyTerminal = [];
$journeyAgent = [];
$presenterTerminal = [];
$presenterAgent = [];
$presenterOf = static function (string $dbState): string {
    $row = ['state' => $dbState];
    return FocusaSpec152eTerminalAgentPaidState::presenter($row)[0];
};
$terminal = $fixture['terminal'];
$agent = $fixture['agent'];
$abandoned = $fixture['abandoned'];
$product = $fixture['product'];
$envelopeCfg = $fixture['envelope'];

// ── Pre-flight negatives: no operation works without a verified registration ──
$bogus = '00000000-0000-0000-0000-000000000000';
okThrows(static fn() => $svc->verifyEmail(array_merge(['registration_uuid' => $bogus, 'code' => $terminal['customer']['verification_code']], $correlation(1, 'pre'))), 'REGISTRATION_NOT_FOUND', 'verify on unknown registration');
okThrows(static fn() => $svc->poll(array_merge(['registration_uuid' => $bogus, 'poll_credential' => 'pollcred_x', 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(1, 'pre'))), 'REGISTRATION_NOT_FOUND', 'poll on unknown registration');
okThrows(static fn() => $svc->receipt(['registration_uuid' => $bogus]), 'REGISTRATION_NOT_FOUND', 'receipt on unknown registration');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => 'attacker_v1', 'origin' => 'https://evil.invalid', 'product_code' => $product['product_code'], 'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'], 'email_prefix_char' => 'x', 'presenter' => $terminal['presenter'], 'install_channel' => $terminal['install_channel'], 'device_public_key' => $terminal['device']['device_public_key_b64'], 'challenge_code' => $terminal['customer']['verification_code']], $correlation(1, 'pre'))), 'FACADE_ORIGIN_DENIED', 'facade spoof denied');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'], 'product_code' => 'focusa_hacker', 'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'], 'email_prefix_char' => 'x', 'presenter' => $terminal['presenter'], 'install_channel' => $terminal['install_channel'], 'device_public_key' => $terminal['device']['device_public_key_b64'], 'challenge_code' => $terminal['customer']['verification_code']], $correlation(1, 'pre'))), 'PRODUCT_MAPPING_REQUIRED', 'unmapped product denied');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'], 'product_code' => $product['product_code'], 'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'], 'email_prefix_char' => 'x', 'presenter' => $terminal['presenter'], 'install_channel' => $terminal['install_channel'], 'device_public_key' => $terminal['device']['device_public_key_b64'], 'challenge_code' => $terminal['customer']['verification_code'], 'price' => '0'], $correlation(1, 'pre'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller price denied at registration');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'], 'product_code' => $product['product_code'], 'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'], 'email_prefix_char' => 'x', 'presenter' => 'attacker_presenter', 'install_channel' => $terminal['install_channel'], 'device_public_key' => $terminal['device']['device_public_key_b64'], 'challenge_code' => $terminal['customer']['verification_code']], $correlation(1, 'pre'))), 'PRESENTER_REQUIRED', 'unknown presenter denied');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'], 'product_code' => $product['product_code'], 'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'], 'email_prefix_char' => 'x', 'presenter' => $terminal['presenter'], 'install_channel' => 'attacker_channel', 'device_public_key' => $terminal['device']['device_public_key_b64'], 'challenge_code' => $terminal['customer']['verification_code']], $correlation(1, 'pre'))), 'INSTALL_CHANNEL_REQUIRED', 'unknown install channel denied');
okThrows(static fn() => $svc->startRegistration(array_merge(['facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'], 'product_code' => $product['product_code'], 'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'], 'email_prefix_char' => 'x', 'presenter' => $terminal['presenter'], 'install_channel' => $terminal['install_channel'], 'device_public_key' => 'bad-key', 'challenge_code' => $terminal['customer']['verification_code']], $correlation(1, 'pre'))), 'DEVICE_PUBLIC_KEY_REQUIRED', 'malformed device key denied');

// ── Session A: interactive terminal paid Focusa ────────────────────────────
$start = $svc->startRegistration(array_merge([
    'facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'],
    'product_code' => $product['product_code'],
    'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'],
    'email_prefix_char' => $terminal['customer']['email_prefix_char'],
    'presenter' => $terminal['presenter'], 'install_channel' => $terminal['install_channel'],
    'device_public_key' => $terminal['device']['device_public_key_b64'],
    'challenge_code' => $terminal['customer']['verification_code'],
], $correlation(2, 'start')));
ok($start['ok'] === true && $start['state'] === 'attempt_created', 'terminal registration created');
ok($start['presenter'] === 'cli_terminal' && $start['install_channel'] === 'terminal', 'terminal presenter bound');
ok($start['masked_email'] === 't***@invalid.example', 'masked email only');
ok($start['customer_created'] === false && $start['entitlement_created'] === false, 'registration creates no customer/entitlement');
ok($start['poll_credential'] !== '' && $start['max_polls'] === 40, 'poll credential issued at start; budget 40');
$regA = $start['registration_uuid'];
$pollCredA = $start['poll_credential'];
$journeyTerminal[] = $start['state'];
$presenterTerminal[] = $presenterOf($start['state']);
$replayStart = $svc->startRegistration(array_merge([
    'facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'],
    'product_code' => $product['product_code'],
    'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'],
    'email_prefix_char' => $terminal['customer']['email_prefix_char'],
    'presenter' => $terminal['presenter'], 'install_channel' => $terminal['install_channel'],
    'device_public_key' => $terminal['device']['device_public_key_b64'],
    'challenge_code' => $terminal['customer']['verification_code'],
], $correlation(2, 'start')));
ok($replayStart['replayed'] === true && $replayStart['idempotent_replay'] === true, 'terminal registration replay is idempotent');
okThrows(static fn() => $svc->startRegistration(array_merge([
    'facade_id' => $terminal['facade_id'], 'origin' => $terminal['origin'],
    'product_code' => $product['product_code'],
    'email_digest' => $terminal['customer']['email_digest'], 'email_domain' => $terminal['customer']['email_domain'],
    'email_prefix_char' => $terminal['customer']['email_prefix_char'],
    'presenter' => $terminal['presenter'], 'install_channel' => $terminal['install_channel'],
    'device_public_key' => $terminal['device']['device_public_key_b64'],
    'challenge_code' => '111111',
], $correlation(2, 'start'))), 'IDEMPOTENCY_CONFLICT', 'idempotency-key reuse with a different request fails');

// Terminal presenters refuse agent pause/resume/status steps.
okThrows(static fn() => $svc->pause(array_merge(['registration_uuid' => $regA], $correlation(3, 'pause'))), 'PAUSE_STEP_DENIED', 'terminal refuses agent pause');
okThrows(static fn() => $svc->resume(array_merge(['registration_uuid' => $regA, 'poll_credential' => $pollCredA], $correlation(3, 'resume'))), 'RESUME_STEP_DENIED', 'terminal refuses agent resume');
okThrows(static fn() => $svc->agentStatus(array_merge(['registration_uuid' => $regA, 'poll_credential' => $pollCredA], $correlation(3, 'status'))), 'AGENT_PRESENTER_REQUIRED', 'agent status requires agent presenter');
okThrows(static fn() => $svc->poll(array_merge(['registration_uuid' => $regA, 'poll_credential' => $pollCredA, 'device_public_key' => $abandoned['device']['device_public_key_b64']], $correlation(3, 'poll'))), 'DEVICE_BINDING_MISMATCH', 'wrong device key denied at poll');
okThrows(static fn() => $svc->poll(array_merge(['registration_uuid' => $regA, 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(3, 'poll'))), 'POLL_CREDENTIAL_REQUIRED', 'poll without credential denied');
okThrows(static fn() => $svc->poll(array_merge(['registration_uuid' => $regA, 'poll_credential' => 'pollcred_wrong', 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(3, 'poll'))), 'POLL_CREDENTIAL_REQUIRED', 'wrong poll credential denied');
okThrows(static fn() => $svc->issueLicense(array_merge(['registration_uuid' => $regA], $correlation(3, 'license'))), 'EDD_ORDER_PENDING', 'no license before order');
okThrows(static fn() => $svc->registerNode(array_merge(['registration_uuid' => $regA, 'node_id' => $terminal['device']['node_id'], 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(3, 'node'))), 'LICENSE_DELIVERY_PENDING', 'no node before delivery');
okThrows(static fn() => $svc->issueLease(array_merge(['registration_uuid' => $regA], $correlation(3, 'lease'))), 'NODE_REQUIRED', 'no lease before node');
okThrows(static fn() => $svc->refund(array_merge(['registration_uuid' => $regA, 'reason' => 'synthetic_proof_cleanup'], $correlation(3, 'refund'))), 'REFUND_STATE_REQUIRED', 'no refund before lease');
okThrows(static fn() => $svc->revealKey(['handle' => 'cred_bogus_0000', 'reveal_key' => true, 'reveal_confirmation' => true, 'now' => ($clock)()]), 'CREDENTIAL_REVEAL_DENIED', 'reveal on unknown handle denied');

// Verification: single-use, attempt-bounded magic code.
okThrows(static fn() => $svc->verifyEmail(array_merge(['registration_uuid' => $regA, 'code' => '000000'], $correlation(4, 'verify'))), 'EMAIL_VERIFICATION_FAILED', 'wrong verification code fails');
$verified = $svc->verifyEmail(array_merge(['registration_uuid' => $regA, 'code' => $terminal['customer']['verification_code']], $correlation(5, 'verify')));
ok($verified['ok'] === true && $verified['state'] === 'email_verified', 'mailbox verified with single-use code');
ok($verified['verification_method'] === 'single_use_magic_code', 'verification method is single-use magic code');
$journeyTerminal[] = $verified['state'];
okThrows(static fn() => $svc->verifyEmail(array_merge(['registration_uuid' => $regA, 'code' => $terminal['customer']['verification_code']], $correlation(6, 'verify'))), 'EMAIL_VERIFICATION_FAILED', 'verification code is single-use (replay fails)');

// Promotion: only after verification.
$promoted = $svc->promote(array_merge(['registration_uuid' => $regA], $correlation(5, 'promote')));
ok($promoted['ok'] === true && $promoted['state'] === 'account_promoted', 'terminal account promoted');
ok((int) $promoted['customer_id'] === (int) $terminal['edd']['customer_id'], 'EDD customer 1494 created');
$journeyTerminal[] = $promoted['state'];
$replayPromote = $svc->promote(array_merge(['registration_uuid' => $regA], $correlation(5, 'promote')));
ok($replayPromote['replayed'] === true && $replayPromote['zero_new_rows'] === true, 'promote replay writes zero rows');

// Checkout link (server-owned facade URL) + bounded poll while payment pending.
okThrows(static fn() => $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regA, 'product_code' => 'focusa_hacker'], $correlation(7, 'intent'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller product denied at checkout');
okThrows(static fn() => $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regA, 'redirect_url' => 'https://evil.invalid'], $correlation(7, 'intent'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller redirect denied at checkout');
$intent = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regA], $correlation(8, 'intent')));
ok($intent['ok'] === true && $intent['state'] === 'checkout_pending', 'checkout intent created');
ok(str_starts_with($intent['branded_checkout_url'], $terminal['origin'] . $terminal['checkout_path']), 'branded facade checkout link');
ok((int) $intent['edd_download_id'] === (int) $product['edd_download_id'] && $intent['edd_price_id'] === $product['edd_price_id'] && $intent['price_usd'] === $product['price_usd'], 'server-owned product/price mapping');
ok($intent['stripe_gateway'] === 'edd_stripe_test_mode' && $intent['card_data_handled_by'] === 'edd_stripe_only', 'EDD Stripe gateway only, no client card data');
$journeyTerminal[] = $intent['state'];
$pollPending = $svc->poll(array_merge(['registration_uuid' => $regA, 'poll_credential' => $pollCredA, 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(9, 'poll')));
ok($pollPending['schema'] === 'focusa.activation.response.v1' && $pollPending['state'] === 'checkout_pending', 'bounded poll while payment pending');
ok($pollPending['poll_count'] === 1 && $pollPending['max_polls'] === 40, 'poll count within budget');
ok(array_key_exists('safe_url', $pollPending) && str_starts_with($pollPending['safe_url'], $terminal['origin'] . $terminal['checkout_path']), 'pending poll carries the safe checkout link');
ok(!array_key_exists('one_time_key_envelope', $pollPending), 'no envelope before delivery');

// Checkout email integrity + completion.
$held = $svc->completePayment(array_merge([
    'registration_uuid' => $regA, 'checkout_email_digest' => $terminal['customer']['invalid_email_digest'],
    'payment_reference_digest' => $terminal['edd']['payment_reference_digest'],
], $correlation(10, 'pay')));
ok($held['ok'] === false && $held['state'] === 'held_unverified' && $held['error'] === 'EDD_ORDER_UNVERIFIED', 'different checkout email holds fulfillment');
ok($held['checkout_email_integrity'] === 'fulfillment_held', 'payment success alone never verifies');
okThrows(static fn() => $svc->issueLicense(array_merge(['registration_uuid' => $regA], $correlation(10, 'license'))), 'EDD_ORDER_PENDING', 'no license while fulfillment is held');
$paid = $svc->completePayment(array_merge([
    'registration_uuid' => $regA, 'checkout_email_digest' => $terminal['customer']['email_digest'],
    'payment_reference_digest' => $terminal['edd']['payment_reference_digest'],
], $correlation(11, 'pay')));
ok($paid['ok'] === true && $paid['state'] === 'complete', 'EDD order completed in test mode');
ok((int) $paid['order_id'] === (int) $terminal['edd']['order_id'] && $paid['checkout_email_integrity'] === 'verified_identity_match', 'one canonical order, verified identity match');
$journeyTerminal[] = 'order_complete';

// EDD Software Licensing issuance: the sole human key.
$license = $svc->issueLicense(array_merge(['registration_uuid' => $regA], $correlation(11, 'license')));
ok($license['ok'] === true && $license['state'] === 'entitlement_issued', 'EDD Software Licensing key issued');
ok((int) $license['edd_license_id'] === (int) $terminal['edd']['edd_license_id'] && $license['source'] === 'edd_software_licensing', 'key source is EDD only');
ok($license['issuance_surface'] === 'edd_authority_only' && $license['duplicate_license'] === false, 'no local/self-issued entitlement');
$journeyTerminal[] = $license['state'];
$keyMaskA = $license['license_key_mask'];
ok(preg_match('/^\*{8}-\*{8}-\*{8}-[0-9A-F]{4}$/', $keyMaskA) === 1, 'issuance returns a masked key only');

// Dual key delivery: transactional email + one-time device-encrypted envelope.
$delivery = $svc->prepareTerminalDelivery(array_merge(['registration_uuid' => $regA], $correlation(12, 'deliver')));
ok($delivery['ok'] === true && $delivery['state'] === 'license_delivery_ready', 'dual delivery prepared');
ok($delivery['channels'] === ['email' => 'sent', 'terminal' => 'ready'], 'email + terminal channels');
ok($delivery['same_canonical_key_both_channels'] === true && $delivery['promotional_content'] === false, 'same canonical key both channels; transactional only');
ok($delivery['key_mask'] === $keyMaskA, 'email channel masks the same canonical key');
ok(preg_match('/^env_[0-9a-f]{32}$/', (string) $delivery['envelope_id']) === 1, 'one-time envelope id bound');
$journeyTerminal[] = $delivery['state'];
$pollCredA = $delivery['poll_credential'];

// Terminal poll delivers the envelope exactly once.
$delivered = $svc->poll(array_merge(['registration_uuid' => $regA, 'poll_credential' => $pollCredA, 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(13, 'poll')));
ok($delivered['state'] === 'terminal_delivered' && $delivered['terminal_delivery_status'] === 'delivered', 'one-time envelope delivered to the terminal device');
ok(array_key_exists('one_time_key_envelope', $delivered) && $delivered['envelope_id'] === $delivery['envelope_id'], 'poll response carries the sealed envelope once');
ok($delivered['license_key_mask'] === $keyMaskA, 'delivery mask matches the canonical key');
$terminalEnvelopeB64 = $delivered['one_time_key_envelope'];
$journeyTerminal[] = $delivered['state'];
okThrows(static fn() => $svc->poll(array_merge(['registration_uuid' => $regA, 'poll_credential' => $pollCredA, 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(14, 'poll'))), 'LICENSE_DELIVERY_FAILED', 'envelope is one-time (second poll fails)');

// Device seam: open the envelope, store the credential, reveal once under consent.
$envelopeA = json_decode((string) base64_decode(strtr($terminalEnvelopeB64, '-_', '+/') . str_repeat('=', (4 - strlen($terminalEnvelopeB64) % 4) % 4)), true, 512, JSON_THROW_ON_ERROR);
$openA = $svc->openEnvelope(['registration_uuid' => $regA, 'envelope' => $envelopeA, 'device_private_key' => $terminal['device']['device_private_key_hex'], 'now' => ($clock)()]);
ok($openA['ok'] === true && $openA['claims_validated'] === true && $openA['one_time'] === true, 'device opens the envelope and validates claims');
ok($openA['license_key_mask'] === $keyMaskA, 'opened envelope carries the same canonical key');
okThrows(static fn() => $svc->openEnvelope(['registration_uuid' => $regA, 'envelope' => $envelopeA, 'device_private_key' => $abandoned['device']['device_private_key_hex'], 'now' => ($clock)()]), 'ENVELOPE_AUTH_FAILED', 'wrong device cannot open the envelope');
okThrows(static fn() => $svc->openEnvelope(['registration_uuid' => $regA, 'envelope' => $envelopeA, 'device_private_key' => $terminal['device']['device_private_key_hex'], 'now' => '2027-01-01T00:00:00Z']), 'ENVELOPE_EXPIRED', 'expired envelope fails closed');
$storedA = $svc->credentialStore(['registration_uuid' => $regA, 'envelope' => $envelopeA, 'device_private_key' => $terminal['device']['device_private_key_hex'], 'now' => ($clock)()] + $correlation(15, 'store'));
ok($storedA['operation'] === 'store' && $storedA['store'] === 'protected_credential_store', 'credential store confirmed');
ok($storedA['revealed'] === false && $storedA['mask'] === $keyMaskA, 'store exposes handle and mask only');
$handleA = $storedA['handle'];
okThrows(static fn() => $svc->revealKey(['handle' => $handleA, 'now' => ($clock)()]), 'CREDENTIAL_REVEAL_DENIED', 'reveal without opt-in denied');
okThrows(static fn() => $svc->revealKey(['handle' => $handleA, 'reveal_key' => true, 'now' => ($clock)()]), 'CREDENTIAL_REVEAL_DENIED', 'reveal without confirmation denied');
$revealA = $svc->revealKey(['handle' => $handleA, 'reveal_key' => true, 'reveal_confirmation' => true, 'now' => ($clock)()]);
ok($revealA['revealed'] === true && $revealA['operation'] === 'reveal', 'explicit customer-controlled reveal succeeds');
$dbKeyA = $db->query("SELECT license_key FROM wp_wpuiai_ta_licenses WHERE edd_license_id = " . (int) $terminal['edd']['edd_license_id'])->fetchColumn();
ok($revealA['license_key'] === $dbKeyA, 'revealed key is the canonical EDD key');
okThrows(static fn() => $svc->revealKey(['handle' => $handleA, 'reveal_key' => true, 'reveal_confirmation' => true, 'now' => ($clock)()]), 'CREDENTIAL_REVEAL_DENIED', 'reveal is one-time (replay denied)');

// Node registration + signed lease.
okThrows(static fn() => $svc->registerNode(array_merge(['registration_uuid' => $regA, 'node_id' => 'bad node id', 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(16, 'node'))), 'NODE_NOT_FOUND', 'malformed node id denied');
okThrows(static fn() => $svc->registerNode(array_merge(['registration_uuid' => $regA, 'node_id' => $terminal['device']['node_id'], 'device_public_key' => $abandoned['device']['device_public_key_b64']], $correlation(16, 'node'))), 'DEVICE_BINDING_MISMATCH', 'node must bind the registered device');
$node = $svc->registerNode(array_merge(['registration_uuid' => $regA, 'node_id' => $terminal['device']['node_id'], 'device_public_key' => $terminal['device']['device_public_key_b64']], $correlation(16, 'node')));
ok($node['ok'] === true && $node['state'] === 'device_registered', 'node registered');
ok($node['binding'] === 'account_and_edd_license' && $node['install_channel_telemetry_only'] === true, 'node bound to account + EDD license; channel telemetry only');
$journeyTerminal[] = $node['state'];
okThrows(static fn() => $svc->issueLease(array_merge(['registration_uuid' => $regA, 'features' => ['all' => true]], $correlation(17, 'lease'))), 'CALLER_CONTROLLED_GRANT_DENIED', 'caller grants denied at lease');
$lease = $svc->issueLease(array_merge(['registration_uuid' => $regA], $correlation(17, 'lease')));
ok($lease['ok'] === true && $lease['state'] === 'lease_issued', 'signed lease issued');
ok((int) $lease['sequence'] === (int) $fixture['lease']['sequence_after_lease'] && $lease['posture'] === 'paid', 'lease sequence 1, paid posture');
ok($lease['authority_key_id'] === $fixture['lease']['lease_key_id'] && $lease['runtime_authorization'] === 'signed_device_bound_lease', 'authority lease key, device-bound runtime authorization');
$journeyTerminal[] = $lease['state'];
$leaseRowA = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ta_leases WHERE lease_uuid = '" . $lease['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);

// Refund cleanup after proof.
$refund = $svc->refund(array_merge(['registration_uuid' => $regA, 'reason' => 'synthetic_proof_cleanup'], $correlation(18, 'refund')));
ok($refund['ok'] === true && $refund['state'] === 'refunded', 'terminal refund processed');
ok((int) $refund['sequence_after'] === (int) $fixture['lease']['sequence_after_refund'] && $refund['refresh_denied'] === true, 'refund increments sequence to 2 and denies refresh');
ok($refund['posture'] === 'recovery_only' && $refund['account_order_delivery_node_evidence_preserved'] === true, 'recovery-only posture, truth preserved');
$journeyTerminal[] = $refund['state'];
$replayRefund = $svc->refund(array_merge(['registration_uuid' => $regA, 'reason' => 'synthetic_proof_cleanup'], $correlation(18, 'refund')));
ok($replayRefund['replayed'] === true && $replayRefund['zero_new_rows'] === true, 'refund replay writes zero rows');
$receiptA = $svc->receipt(['registration_uuid' => $regA]);
ok($receiptA['schema'] === 'focusa.spec152e.terminal_agent_receipt.v1' && $receiptA['state'] === 'refunded', 'terminal receipt reflects refund');
ok($receiptA['install_site_authority'] === 'none' && $receiptA['spec158'] === 'excluded', 'no install-site authority; spec 158 excluded');
ok(preg_match('/^[0-9a-f]{64}$/', (string) $receiptA['receipt_sha256']) === 1, 'terminal receipt carries an immutable sha256 handle');
$presenterTerminal[] = $presenterOf('checkout_pending');
$presenterTerminal[] = $presenterOf('terminal_delivered');
$presenterTerminal[] = $presenterOf('lease_issued');
$presenterTerminal[] = $presenterOf('refunded');

// ── Session B: agent_json paid Focusa with pause/resume ────────────────────
$startB = $svc->startRegistration(array_merge([
    'facade_id' => $agent['facade_id'], 'origin' => $agent['origin'],
    'product_code' => $product['product_code'],
    'email_digest' => $agent['customer']['email_digest'], 'email_domain' => $agent['customer']['email_domain'],
    'email_prefix_char' => $agent['customer']['email_prefix_char'],
    'presenter' => $agent['presenter'], 'install_channel' => $agent['install_channel'],
    'device_public_key' => $agent['device']['device_public_key_b64'],
    'challenge_code' => $agent['customer']['verification_code'],
], $correlation(20, 'start')));
ok($startB['ok'] === true && $startB['state'] === 'attempt_created', 'agent registration created');
ok($startB['presenter'] === 'agent_json' && $startB['install_channel'] === 'agent', 'agent presenter bound');
$regB = $startB['registration_uuid'];
$journeyAgent[] = $startB['state'];
$statusB1 = $svc->agentStatus(array_merge(['registration_uuid' => $regB, 'poll_credential' => $startB['poll_credential']], $correlation(21, 'status')));
ok($statusB1['schema'] === 'focusa.agent_activation_envelope.v1' && $statusB1['state'] === 'email_verification_pending', 'agent transcript: email verification pending');
ok($statusB1['human_action_required'] === true && $statusB1['human_action'] === 'enter_verification_code', 'agent never invents the code; human must enter it');
ok($statusB1['key_present'] === false && $statusB1['key_visible'] === false, 'no key material in the agent transcript');
ok($statusB1['masked_email'] === 'a***@invalid.example', 'agent transcript masks the email');
$presenterAgent[] = $statusB1['state'];
$verifyB = $svc->verifyEmail(array_merge(['registration_uuid' => $regB, 'code' => $agent['customer']['verification_code']], $correlation(21, 'verify')));
ok($verifyB['ok'] === true && $verifyB['state'] === 'email_verified', 'agent mailbox verified');
$journeyAgent[] = $verifyB['state'];
$statusB2 = $svc->agentStatus(array_merge(['registration_uuid' => $regB, 'poll_credential' => $startB['poll_credential']], $correlation(22, 'status')));
ok($statusB2['state'] === 'selection_required' && $statusB2['human_action'] === 'select_offer', 'agent presents offer selection to the human');
$presenterAgent[] = $statusB2['state'];
$promoteB = $svc->promote(array_merge(['registration_uuid' => $regB], $correlation(22, 'promote')));
ok($promoteB['ok'] === true && (int) $promoteB['customer_id'] === (int) $agent['edd']['customer_id'], 'agent account promoted (customer 1660)');
$journeyAgent[] = $promoteB['state'];
$statusB3 = $svc->agentStatus(array_merge(['registration_uuid' => $regB, 'poll_credential' => $startB['poll_credential']], $correlation(23, 'status')));
ok($statusB3['state'] === 'checkout_required' && $statusB3['human_action'] === 'open_checkout_url', 'agent presents checkout required to the human');
$presenterAgent[] = $statusB3['state'];
$intentB = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regB], $correlation(23, 'intent')));
ok($intentB['ok'] === true && $intentB['state'] === 'checkout_pending', 'agent checkout intent created');
$journeyAgent[] = $intentB['state'];
$statusB4 = $svc->agentStatus(array_merge(['registration_uuid' => $regB, 'poll_credential' => $startB['poll_credential']], $correlation(24, 'status')));
ok($statusB4['state'] === 'payment_pending' && array_key_exists('safe_url', $statusB4), 'agent presents the safe checkout link');
ok(str_starts_with($statusB4['safe_url'], $agent['origin'] . $agent['checkout_path']), 'safe_url is the facade checkout link');
$pollB1 = $svc->poll(array_merge(['registration_uuid' => $regB, 'poll_credential' => $startB['poll_credential']], $correlation(25, 'poll')));
ok($pollB1['state'] === 'payment_pending' && $pollB1['poll_count'] === 1, 'agent bounded poll: payment pending');
ok(array_key_exists('safe_url', $pollB1) && $pollB1['key_visible'] === false, 'agent poll carries the link, never the key');
$presenterAgent[] = $pollB1['state'];
$pollB2 = $svc->poll(array_merge(['registration_uuid' => $regB, 'poll_credential' => $startB['poll_credential']], $correlation(26, 'poll')));
ok($pollB2['state'] === 'payment_pending' && $pollB2['poll_count'] === 2, 'agent polls within budget while the human pays');
// Pause: human steps away; the protected poll credential rotates.
$paused = $svc->pause(array_merge(['registration_uuid' => $regB], $correlation(26, 'pause')));
ok($paused['ok'] === true && $paused['paused'] === true, 'agent session paused');
ok($paused['poll_credential'] !== '' && $paused['poll_credential'] !== $startB['poll_credential'], 'pause rotates the protected poll credential');
$pollCredB = $paused['poll_credential'];
okThrows(static fn() => $svc->pause(array_merge(['registration_uuid' => $regB], $correlation(27, 'pause'))), 'PAUSE_ALREADY_PAUSED', 'double pause denied');
okThrows(static fn() => $svc->poll(array_merge(['registration_uuid' => $regB, 'poll_credential' => $pollCredB], $correlation(27, 'poll'))), 'SESSION_PAUSED', 'poll while paused denied');
// The human completes payment at the checkout link (authority hook).
$paidB = $svc->completePayment(array_merge([
    'registration_uuid' => $regB, 'checkout_email_digest' => $agent['customer']['email_digest'],
    'payment_reference_digest' => $agent['edd']['payment_reference_digest'],
], $correlation(27, 'pay')));
ok($paidB['ok'] === true && (int) $paidB['order_id'] === (int) $agent['edd']['order_id'], 'agent order completed (9796)');
$journeyAgent[] = 'order_complete';
okThrows(static fn() => $svc->resume(array_merge(['registration_uuid' => $regB], $correlation(28, 'resume'))), 'POLL_CREDENTIAL_REQUIRED', 'resume requires the re-supplied protected credential');
okThrows(static fn() => $svc->resume(array_merge(['registration_uuid' => $regB, 'poll_credential' => 'pollcred_wrong'], $correlation(28, 'resume'))), 'POLL_CREDENTIAL_REQUIRED', 'resume with a wrong credential denied');
$resumed = $svc->resume(array_merge(['registration_uuid' => $regB, 'poll_credential' => $pollCredB], $correlation(28, 'resume')));
ok($resumed['schema'] === 'focusa.agent_activation_envelope.v1' && $resumed['state'] === 'license_delivery_ready', 'agent resumes to license delivery ready');
ok($resumed['poll_count'] === 3 && $resumed['key_visible'] === false, 'resume is a bounded poll; key stays masked');
$presenterAgent[] = $resumed['state'];
$licenseB = $svc->issueLicense(array_merge(['registration_uuid' => $regB], $correlation(28, 'license')));
ok($licenseB['ok'] === true && (int) $licenseB['edd_license_id'] === (int) $agent['edd']['edd_license_id'], 'agent EDD key issued (7440)');
$journeyAgent[] = $licenseB['state'];
$keyMaskB = $licenseB['license_key_mask'];
$deliveryB = $svc->prepareTerminalDelivery(array_merge(['registration_uuid' => $regB], $correlation(29, 'deliver')));
ok($deliveryB['ok'] === true && $deliveryB['state'] === 'license_delivery_ready', 'agent dual delivery prepared');
ok($deliveryB['key_mask'] === $keyMaskB && $deliveryB['same_canonical_key_both_channels'] === true, 'agent delivery carries the same canonical key');
$journeyAgent[] = $deliveryB['state'];
$agentTranscript = $svc->agentStatus(array_merge(['registration_uuid' => $regB, 'poll_credential' => $deliveryB['poll_credential']], $correlation(30, 'status')));
ok($agentTranscript['state'] === 'license_delivery_ready' && $agentTranscript['key_present'] === true && $agentTranscript['key_visible'] === false, 'agent transcript: key present but never visible');
foreach (['email', 'normalized_email', 'raw_email', 'full_license_key', 'one_time_key_envelope', 'lease_envelope', 'poll_credential', 'poll_credential_hash', 'verification_hash', 'server_credential', 'signing_key', 'card_pan', 'card_expiry', 'card_cvc', 'edd_internal_record'] as $forbiddenField) {
    ok(!array_key_exists($forbiddenField, $agentTranscript), "agent transcript never contains {$forbiddenField}");
}
$envelopeIdB = $deliveryB['envelope_id'];
$envelopeRowBraw = $db->query("SELECT envelope_payload FROM wp_wpuiai_ta_envelopes WHERE envelope_id = '" . $envelopeIdB . "'")->fetchColumn();
$envelopeB = json_decode((string) $envelopeRowBraw, true, 512, JSON_THROW_ON_ERROR);
$openB = $svc->openEnvelope(['registration_uuid' => $regB, 'envelope_id' => $envelopeIdB, 'envelope' => $envelopeB, 'device_private_key' => $agent['device']['device_private_key_hex'], 'now' => ($clock)()]);
ok($openB['ok'] === true && $openB['envelope_id'] === $envelopeIdB, 'agent device opens its one-time envelope out-of-band');
$storedB = $svc->credentialStore(['registration_uuid' => $regB, 'envelope_id' => $envelopeIdB, 'device_private_key' => $agent['device']['device_private_key_hex'], 'now' => ($clock)()] + $correlation(30, 'store'));
ok($storedB['operation'] === 'store' && $storedB['revealed'] === false, 'agent credential store confirmed');
$handleB = $storedB['handle'];
okThrows(static fn() => $svc->openEnvelope(['registration_uuid' => $regB, 'envelope_id' => $envelopeIdB, 'device_private_key' => $agent['device']['device_private_key_hex'], 'now' => ($clock)()]), 'ENVELOPE_ALREADY_CONSUMED', 'one-time envelope cannot be consumed twice');
$revealB = $svc->revealKey(['handle' => $handleB, 'reveal_key' => true, 'reveal_confirmation' => true, 'now' => ($clock)()]);
ok($revealB['revealed'] === true, 'agent human reveals the key once under explicit consent');
$dbKeyB = $db->query("SELECT license_key FROM wp_wpuiai_ta_licenses WHERE edd_license_id = " . (int) $agent['edd']['edd_license_id'])->fetchColumn();
ok($revealB['license_key'] === $dbKeyB, 'agent revealed key is the canonical EDD key');
$nodeB = $svc->registerNode(array_merge(['registration_uuid' => $regB, 'node_id' => $agent['device']['node_id'], 'device_public_key' => $agent['device']['device_public_key_b64']], $correlation(31, 'node')));
ok($nodeB['ok'] === true && $nodeB['state'] === 'device_registered', 'agent node registered');
$journeyAgent[] = $nodeB['state'];
$leaseB = $svc->issueLease(array_merge(['registration_uuid' => $regB], $correlation(31, 'lease')));
ok($leaseB['ok'] === true && (int) $leaseB['sequence'] === 1 && $leaseB['posture'] === 'paid', 'agent signed lease issued, sequence 1');
$journeyAgent[] = $leaseB['state'];
$leaseRowB = $db->query("SELECT payload_b64, signature_b64 FROM wp_wpuiai_ta_leases WHERE lease_uuid = '" . $leaseB['lease_uuid'] . "'")->fetch(PDO::FETCH_ASSOC);
$activatedB = $svc->poll(array_merge(['registration_uuid' => $regB, 'poll_credential' => $deliveryB['poll_credential']], $correlation(32, 'poll')));
ok($activatedB['state'] === 'activated' && $activatedB['terminal'] === true, 'agent session resumes activated');
ok($activatedB['human_action_required'] === false && $activatedB['next_action'] === 'none', 'activated session requires no human action');
$presenterAgent[] = $activatedB['state'];
$refundB = $svc->refund(array_merge(['registration_uuid' => $regB, 'reason' => 'synthetic_proof_cleanup'], $correlation(32, 'refund')));
ok($refundB['ok'] === true && (int) $refundB['sequence_after'] === 2 && $refundB['posture'] === 'recovery_only', 'agent refund increments sequence and revokes');
$journeyAgent[] = $refundB['state'];
$receiptB = $svc->receipt(['registration_uuid' => $regB]);
ok($receiptB['state'] === 'refunded' && preg_match('/^[0-9a-f]{64}$/', (string) $receiptB['receipt_sha256']) === 1, 'agent receipt redacted with immutable handle');
$recoveryB = $svc->poll(array_merge(['registration_uuid' => $regB, 'poll_credential' => $deliveryB['poll_credential']], $correlation(33, 'poll')));
ok($recoveryB['state'] === 'recovery_only' && $recoveryB['retry_posture'] === 'none', 'refunded agent session settles recovery-only');
$presenterAgent[] = $recoveryB['state'];

// ── Session C: abandoned agent polls past the budget, cancels fail-closed ──
$startC = $svc->startRegistration(array_merge([
    'facade_id' => $abandoned['facade_id'], 'origin' => $abandoned['origin'],
    'product_code' => $product['product_code'],
    'email_digest' => $abandoned['customer']['email_digest'], 'email_domain' => $abandoned['customer']['email_domain'],
    'email_prefix_char' => $abandoned['customer']['email_prefix_char'],
    'presenter' => $abandoned['presenter'], 'install_channel' => $abandoned['install_channel'],
    'device_public_key' => $abandoned['device']['device_public_key_b64'],
    'challenge_code' => $abandoned['customer']['verification_code'],
], $correlation(40, 'start')));
$regC = $startC['registration_uuid'];
$verifyC = $svc->verifyEmail(array_merge(['registration_uuid' => $regC, 'code' => $abandoned['customer']['verification_code']], $correlation(40, 'verify')));
ok($verifyC['ok'] === true, 'abandoned session verifies');
$promoteC = $svc->promote(array_merge(['registration_uuid' => $regC], $correlation(40, 'promote')));
ok((int) $promoteC['customer_id'] === (int) $abandoned['edd']['customer_id'], 'abandoned session promoted (customer 1786)');
$intentC = $svc->createCheckoutIntent(array_merge(['registration_uuid' => $regC], $correlation(40, 'intent')));
ok($intentC['ok'] === true && $intentC['state'] === 'checkout_pending', 'abandoned session reaches payment pending');
$exhausted = null;
$maxSeen = 0;
for ($i = 1; $i <= 42; $i++) {
    $pollC = $svc->poll(array_merge(['registration_uuid' => $regC, 'poll_credential' => $startC['poll_credential']], $correlation(40 + $i, 'poll')));
    $maxSeen = max($maxSeen, (int) $pollC['poll_count']);
    if ($pollC['state'] === 'recovery_only') {
        $exhausted = $pollC;
        break;
    }
    ok($i <= 40, 'bounded poll stays within budget');
}
ok($exhausted !== null && $exhausted['state'] === 'recovery_only', 'poll budget exhaustion cancels to recovery_only');
ok($maxSeen === 40, 'budget consumed exactly max_polls=40');
$cCounts = [
    'orders' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_ta_orders WHERE account_uuid = (SELECT account_uuid FROM wp_wpuiai_ta_registrations WHERE registration_uuid = '{$regC}')")->fetchColumn(),
    'licenses' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_ta_licenses WHERE customer_id = " . (int) $abandoned['edd']['customer_id'])->fetchColumn(),
    'nodes' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_ta_nodes WHERE account_uuid = (SELECT account_uuid FROM wp_wpuiai_ta_registrations WHERE registration_uuid = '{$regC}')")->fetchColumn(),
    'leases' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_ta_leases WHERE account_uuid = (SELECT account_uuid FROM wp_wpuiai_ta_registrations WHERE registration_uuid = '{$regC}')")->fetchColumn(),
    'refunds' => (int) $db->query("SELECT COUNT(*) FROM wp_wpuiai_ta_refunds WHERE order_id IN (SELECT order_id FROM wp_wpuiai_ta_orders WHERE account_uuid = (SELECT account_uuid FROM wp_wpuiai_ta_registrations WHERE registration_uuid = '{$regC}'))")->fetchColumn(),
];
ok($cCounts === ['orders' => 0, 'licenses' => 0, 'nodes' => 0, 'leases' => 0, 'refunds' => 0], 'exhausted session created zero order/license/node/lease/refund rows');

// ── Final counts, sequences, and receipts ──────────────────────────────────
$final = $counts();
$vectors = $fixture['journey']['journal_vectors'];
ok($final['registrations'] === $vectors['registrations'], 'registration row count pinned');
ok($final['identities'] === $vectors['identities'] && $final['accounts'] === $vectors['accounts'], 'identity/account row counts pinned');
ok($final['orders'] === $vectors['orders'] && $final['order_items'] === $vectors['order_items'], 'order/order-item row counts pinned');
ok($final['licenses'] === $vectors['licenses'] && $final['deliveries'] === $vectors['deliveries'], 'license/delivery row counts pinned');
ok($final['envelopes'] === $vectors['envelopes'] && $final['credential_stores'] === $vectors['credential_stores'], 'envelope/credential-store row counts pinned');
ok($final['nodes'] === $vectors['nodes'] && $final['leases'] === $vectors['leases'], 'node/lease row counts pinned');
ok($final['sequences'] === $vectors['sequences'] && $final['refunds'] === $vectors['refunds'], 'sequence/refund row counts pinned');
ok($final['journal'] === $vectors['journal_events'], 'journal event count pinned');
$seqA = (int) $db->query("SELECT current_sequence FROM wp_wpuiai_ta_sequences WHERE account_uuid = (SELECT account_uuid FROM wp_wpuiai_ta_registrations WHERE registration_uuid = '{$regA}') AND product_code = '" . $product['product_code'] . "'")->fetchColumn();
$seqB = (int) $db->query("SELECT current_sequence FROM wp_wpuiai_ta_sequences WHERE account_uuid = (SELECT account_uuid FROM wp_wpuiai_ta_registrations WHERE registration_uuid = '{$regB}') AND product_code = '" . $product['product_code'] . "'")->fetchColumn();
ok($seqA === 2 && $seqB === 2, 'both session sequence ledgers end at 2 after refund');
$preserved = $schema->preserveForRollback('2026-08-09T06:00:00Z', ['source' => 'terminal_agent_paid_rollback']);
ok($preserved['action'] === 'preserve', 'rollback is preservation-only');

$leaseEnvelopeTerminal = [
    'payload_b64' => $leaseRowA['payload_b64'],
    'signature_b64' => $leaseRowA['signature_b64'],
    'lease_public_key_b64' => $fixture['lease']['lease_public_key_b64'],
    'domain' => $fixture['lease']['domain'],
];
$leaseEnvelopeAgent = [
    'payload_b64' => $leaseRowB['payload_b64'],
    'signature_b64' => $leaseRowB['signature_b64'],
    'lease_public_key_b64' => $fixture['lease']['lease_public_key_b64'],
    'domain' => $fixture['lease']['domain'],
];
$envelopeRowB = $db->query("SELECT envelope_payload FROM wp_wpuiai_ta_envelopes WHERE envelope_id = '" . $envelopeIdB . "'")->fetchColumn();
$agentEnvelopeB64 = rtrim(strtr(base64_encode((string) $envelopeRowB), '+/', '-_'), '=');

$summary = [
    'schema' => 'focusa.spec152e.terminal_agent_paid_test.v1',
    'positive_checks' => $positive,
    'negative_checks' => $negative,
    'journey_states_terminal' => $journeyTerminal,
    'journey_states_agent' => $journeyAgent,
    'presenter_states_terminal' => $presenterTerminal,
    'presenter_states_agent' => $presenterAgent,
    'counts' => $final,
    'sequences' => ['terminal' => $seqA, 'agent' => $seqB],
    'key_masks' => ['terminal' => $keyMaskA, 'agent' => $keyMaskB],
    'receipt_sha256_terminal' => $receiptA['receipt_sha256'],
    'receipt_sha256_agent' => $receiptB['receipt_sha256'],
    'lease_envelope_terminal' => $leaseEnvelopeTerminal,
    'lease_envelope_agent' => $leaseEnvelopeAgent,
    'terminal_envelope' => $terminalEnvelopeB64,
    'agent_envelope_delivery' => $agentEnvelopeB64,
    'agent_transcript_sample' => $agentTranscript,
    'poll_exhausted' => ['max_polls' => 40, 'settled' => 'recovery_only', 'poll_count_at_exhaustion' => $maxSeen],
    'reveal' => ['terminal' => ['consumed' => true], 'agent' => ['consumed' => true]],
    'result' => 'passed_fail_closed',
];
fwrite(STDOUT, json_encode($summary, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n");
"""


def run_harness() -> str:
    if not PHP:
        raise AssertionError("FAIL: php is required to execute the terminal/agent paid journey")
    with tempfile.TemporaryDirectory() as tmp:
        harness_path = Path(tmp) / "terminal_agent_harness.php"
        harness_path.write_text(HARNESS, encoding="utf-8")
        proc = subprocess.run(
            [PHP, str(harness_path), str(LEASE_CONTRACT), str(ENVELOPE_CONTRACT), str(CONTRACT), str(FIXTURE)],
            capture_output=True, text=True, timeout=240,
        )
        if proc.returncode != 0:
            raise AssertionError(f"FAIL: php harness exited {proc.returncode}: {proc.stderr[:2000]}")
        return proc.stdout.strip()


first = run_harness()
second = run_harness()
expect(first == second, "harness output is byte-identical across runs (replayable)")
result = json.loads(first)
expect(result["result"] == "passed_fail_closed", "harness passed fail-closed")

fixture_raw = FIXTURE.read_text(encoding="utf-8")
fixture = json.loads(fixture_raw)
contract_raw = CONTRACT.read_text(encoding="utf-8")
envelope_contract_raw = ENVELOPE_CONTRACT.read_text(encoding="utf-8")

# ── Signed leases: independent Ed25519 verification (byte-compatible) ───────

lease_domain = fixture["lease"]["domain"].encode()
for session_key, edd in (("terminal", fixture["terminal"]["edd"]), ("agent", fixture["agent"]["edd"])):
    lease_env = result[f"lease_envelope_{session_key}"]
    payload_bytes = base64.b64decode(lease_env["payload_b64"])
    signature = base64.b64decode(lease_env["signature_b64"])
    public_key = Ed25519PublicKey.from_public_bytes(base64.b64decode(lease_env["lease_public_key_b64"]))
    try:
        public_key.verify(signature, lease_domain + payload_bytes)
        expect(True, f"lease signature verifies (Ed25519, domain-separated) for {session_key}")
    except InvalidSignature as exc:  # pragma: no cover - only on signature breakage
        raise AssertionError(f"FAIL: lease signature did not verify for {session_key}: {exc}")
    lease_payload = json.loads(payload_bytes)
    expect(lease_payload["schema"] == "focusa.authority_lease.v1", f"lease payload schema for {session_key}")
    expect(lease_payload["posture"] == "paid" and lease_payload["product_code"] == "focusa_operator_lifetime_v1", f"paid Focusa posture for {session_key}")
    expect(lease_payload["sequence"] == 1 and lease_payload["status"] == "active", f"lease sequence 1 active for {session_key}")
    expect(lease_payload["customer_id"] == edd["customer_id"] and lease_payload["order_id"] == edd["order_id"] and lease_payload["edd_license_id"] == edd["edd_license_id"], f"lease binds customer/order/license truth for {session_key}")
    expect(re.fullmatch(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", lease_payload["node_id"]) is not None and "device_public_key_hash" in lease_payload, f"lease binds the registered node/device for {session_key}")

# ── One-time terminal envelopes: X25519 open proves the same canonical key ──

envelope_info = fixture["envelope"]["info"].encode()
for session_key in ("terminal", "agent"):
    session = fixture[session_key]
    device_private = bytes.fromhex(session["device"]["device_private_key_hex"])
    sealed = b64url_decode(result[f"{session_key}_envelope_delivery"] if session_key == "agent" else result["terminal_envelope"])
    envelope = json.loads(sealed)
    try:
        plaintext = open_terminal_envelope(device_private, envelope, envelope_info)
        expect(True, f"one-time envelope opens with the device X25519 key for {session_key}")
    except InvalidTag as exc:  # pragma: no cover - only on crypto breakage
        raise AssertionError(f"FAIL: envelope auth failed for {session_key}: {exc}")
    claims = json.loads(plaintext)
    expected_key = derive_expected_key(session["edd"]["edd_license_id"], session["edd"]["order_id"])
    expect(claims["schema"] == "focusa.spec152e.terminal_delivery_envelope.v1", f"envelope claims schema for {session_key}")
    expect(claims["one_time"] is True, f"envelope is one-time for {session_key}")
    expect(claims["license_key"] == expected_key, f"terminal-delivered key is the canonical EDD key for {session_key}")
    expect(claims["edd_license_id"] == session["edd"]["edd_license_id"] and claims["customer_id"] == session["edd"]["customer_id"], f"envelope binds license/customer truth for {session_key}")
    expect(claims["product_code"] == "focusa_operator_lifetime_v1", f"envelope binds the product for {session_key}")
    expect(claims["expires_at"] > claims["issued_at"], f"envelope lifetime bounded for {session_key}")
    # Same canonical key on the email channel (dual key delivery).
    expect(result["key_masks"][session_key] == "********-********-********-" + expected_key[-4:], f"email and terminal channels carry the same canonical key mask for {session_key}")

# ── Masked agent transcript: no forbidden fields, no raw material ──────────

transcript = result["agent_transcript_sample"]
expect(transcript["schema"] == "focusa.agent_activation_envelope.v1", "agent transcript schema")
expect(transcript["state"] == "license_delivery_ready", "agent transcript state")
expect(transcript["key_present"] is True and transcript["key_visible"] is False, "agent transcript masks the key")
for field in (
    "email", "normalized_email", "raw_email", "full_license_key", "one_time_key_envelope",
    "lease_envelope", "poll_credential", "poll_credential_hash", "verification_hash",
    "server_credential", "signing_key", "card_pan", "card_expiry", "card_cvc", "edd_internal_record",
):
    expect_negative(field not in transcript, f"agent transcript has no {field}")
expect(transcript["masked_email"] == "a***@invalid.example", "agent transcript masks the email")

# ── Fixture structure and expectations ─────────────────────────────────────

expect(fixture["schema"] == "focusa.spec152e.terminal_agent_paid_fixture.v1", "fixture schema id")
expect(fixture["fixture_id"] == "focusa-vbcqu.20.13.57", "fixture_id")
expect(fixture["fixture_kind"] == "public_synthetic_nonproduction", "public synthetic non-production fixture")
expect(fixture["authority"]["canonical"] == "WPUIAI.com EDD", "canonical authority")
expect(fixture["authority"]["new_issuance"] == "edd_authority_only", "new issuance edd only")
expect(fixture["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facade presenter/proxy only")
expect(fixture["authority"]["install_site_authority"] == "none", "no install-site authority")
expect(fixture["authority"]["spec158"] == "excluded", "spec158 excluded")
expect(fixture["redaction"] == {
    "raw_email": "absent", "raw_key": "absent", "payment_id_stored": False,
    "secret_material": "absent", "poll_credential": "hash_only_at_rest",
    "receipt": "masked_email_and_key_mask_only",
}, "redaction posture")
expect(fixture["product"]["product_code"] == "focusa_operator_lifetime_v1", "public product code")
expect(fixture["poll"]["default_max_polls"] == 40, "bounded poll default 40")
expect(fixture["poll"]["envelope_schema"] == "focusa.agent_activation_envelope.v1", "agent envelope schema")
expect(fixture["poll"]["timeout_settles_fail_closed"] == "cancel_to_recovery_only", "timeout settles recovery-only")
expect(fixture["envelope"]["algorithm"] == "X25519+HKDF-SHA256+AES-256-GCM", "envelope algorithm")
expect(fixture["envelope"]["info"] == "focusa.spec152e.terminal_delivery_envelope.v1\0hkdf", "HKDF info domain string")
expect(fixture["lease"]["domain"] == "FOCUSA-AUTHORITY-LEASE-V1\0", "lease domain separation")
expect(fixture["journey"]["expectations"]["pending_email_never_promotes"] is True, "pending email never promotes")
expect(fixture["journey"]["expectations"]["no_local_self_issued_entitlement"] is True, "no local/self-issued entitlement")
expect(fixture["journey"]["expectations"]["no_independent_facade_authority"] is True, "no independent facade authority")
expect(fixture["journey"]["expectations"]["client_price_grant_rejected"] is True, "client price/grants rejected")
expect(fixture["journey"]["expectations"]["checkout_email_mismatch_holds_fulfillment"] is True, "checkout email mismatch holds fulfillment")
expect(fixture["journey"]["expectations"]["agent_never_invents_email_code_consent_payment_license"] is True, "agent never invents human actions")
expect(fixture["journey"]["expectations"]["agent_transcript_masked_no_raw_key_or_envelope"] is True, "agent transcript masked")
expect(fixture["journey"]["expectations"]["bounded_poll_within_budget"] is True, "bounded poll within budget")
expect(fixture["journey"]["expectations"]["poll_budget_exhaustion_settles_recovery_only"] is True, "poll budget exhaustion settles recovery-only")
expect(fixture["journey"]["expectations"]["pause_resume_agent_only"] is True, "pause/resume agent only")
expect(fixture["journey"]["expectations"]["terminal_registrations_refuse_resume_steps"] is True, "terminal registrations refuse resume steps")
expect(fixture["journey"]["expectations"]["explicit_key_reveal_requires_optin_and_confirmation"] is True, "explicit key reveal policy")
expect(fixture["journey"]["expectations"]["credential_store_protected_handle_only"] is True, "credential store protected handle only")
expect(fixture["journey"]["expectations"]["one_time_envelope_never_in_agent_transcript"] is True, "one-time envelope never in agent transcript")
expect(fixture["journey"]["expectations"]["same_canonical_key_email_and_terminal"] is True, "same canonical key both channels")
expect(fixture["journey"]["expectations"]["signed_lease_verifies"] is True, "signed lease verifies")
expect(fixture["journey"]["expectations"]["refund_increments_sequence_and_preserves_truth"] is True, "refund increments sequence and preserves truth")
expect(fixture["journey"]["expectations"]["receipt_redacted"] is True, "receipt redacted")
expect(fixture["journey"]["expectations"]["spec158_excluded"] is True, "spec158 excluded")

# ── Contract static invariants ─────────────────────────────────────────────

expect("final class FocusaSpec152eTerminalAgentPaidMigration" in contract_raw, "migration class")
expect("final class FocusaSpec152eTerminalAgentPaidService" in contract_raw, "service class")
expect("focusa.spec152e.terminal_agent_paid_activation.v1" in contract_raw, "contract schema id")
expect("focusa.agent_activation_envelope.v1" in contract_raw, "agent envelope schema")
expect("focusa.activation.response.v1" in contract_raw, "terminal response schema")
for table in (
    "wpuiai_ta_registrations", "wpuiai_ta_identities", "wpuiai_ta_accounts", "wpuiai_ta_orders",
    "wpuiai_ta_order_items", "wpuiai_ta_licenses", "wpuiai_ta_deliveries", "wpuiai_ta_envelopes",
    "wpuiai_ta_credential_stores", "wpuiai_ta_nodes", "wpuiai_ta_leases", "wpuiai_ta_sequences",
    "wpuiai_ta_refunds", "wpuiai_ta_journal",
):
    expect(table in contract_raw, f"table {table}")
for method in (
    "function startRegistration", "function verifyEmail", "function promote",
    "function createCheckoutIntent", "function completePayment", "function issueLicense",
    "function prepareTerminalDelivery", "function poll", "function agentStatus",
    "function pause", "function resume", "function openEnvelope",
    "function credentialStore", "function revealKey", "function registerNode",
    "function issueLease", "function refund", "function receipt",
):
    expect(method in contract_raw, f"method {method}")
for code in (
    "FACADE_ORIGIN_DENIED", "PRODUCT_MAPPING_REQUIRED", "CALLER_CONTROLLED_GRANT_DENIED",
    "PRESENTER_REQUIRED", "INSTALL_CHANNEL_REQUIRED", "DEVICE_PUBLIC_KEY_REQUIRED",
    "EMAIL_VERIFICATION_REQUIRED", "EMAIL_VERIFICATION_FAILED", "EMAIL_VERIFICATION_EXPIRED",
    "EDD_CHECKOUT_REQUIRED", "EDD_ORDER_UNVERIFIED", "EDD_ORDER_PENDING",
    "EDD_LICENSE_PENDING", "EDD_LICENSE_UNUSABLE", "NODE_NOT_FOUND", "NODE_REQUIRED",
    "REFUND_STATE_REQUIRED", "REQUEST_ID_REQUIRED", "IDEMPOTENCY_KEY_REQUIRED",
    "IDEMPOTENCY_CONFLICT", "REGISTRATION_NOT_FOUND", "LICENSE_DELIVERY_PENDING",
    "LICENSE_DELIVERY_FAILED", "POLL_CREDENTIAL_REQUIRED", "POLL_CREDENTIAL_EXPIRED",
    "SESSION_PAUSED", "PAUSE_STEP_DENIED", "PAUSE_STATE_DENIED",
    "PAUSE_ALREADY_PAUSED", "RESUME_STEP_DENIED", "RESUME_STATE_DENIED",
    "AGENT_PRESENTER_REQUIRED", "CREDENTIAL_REVEAL_DENIED", "CREDENTIAL_REVEAL_EXPIRED",
    "ENVELOPE_ALREADY_CONSUMED", "DELIVERY_ALREADY_PREPARED", "DEVICE_BINDING_MISMATCH",
):
    expect(code in contract_raw, f"fail-closed code {code}")
# Envelope crypto/claims codes come from the canonical terminal-delivery-envelope
# contract (required at runtime by this journey) and are exercised here.
for code in ("ENVELOPE_FORMAT_DENIED", "ENVELOPE_AUTH_FAILED", "ENVELOPE_BINDING_MISMATCH",
             "ENVELOPE_EXPIRED", "ENVELOPE_DEVICE_KEY_DENIED"):
    expect(code in envelope_contract_raw, f"envelope fail-closed code {code}")
expect("PRODUCT_MAPPING" in contract_raw and "edd_download_id" in contract_raw, "server-owned product mapping")
expect("FACADE_ALLOWLIST" in contract_raw and "focusa_install_v1" in contract_raw, "registered facade allowlist")
expect("FocusaSpec152eEd25519Signer::LEASE_DOMAIN" in contract_raw, "canonical lease-signing domain")
expect("FocusaSpec152eTerminalEnvelopeCrypto::seal" in contract_raw, "canonical terminal envelope seal")
expect("hash_equals" in contract_raw, "constant-time digest comparison")
expect("poll_count" in contract_raw and "max_polls" in contract_raw, "bounded poll fields")
expect("poll_budget_exhausted" in contract_raw, "poll budget exhaustion settles fail-closed")
expect("reveal_key" in contract_raw and "reveal_confirmation" in contract_raw, "explicit reveal opt-in + confirmation")
expect("spec158" not in contract_raw or "excluded" in contract_raw, "spec158 excluded asserted")
expect("install_site_authority" in contract_raw, "install-site authority posture asserted")
expect("INSERT OR IGNORE" not in contract_raw or "preserveForRollback" in contract_raw, "preservation seam present")
# Preservation-only: no destructive path may exist anywhere in the contract.
for forbidden in ("DELETE FROM", "TRUNCATE", "DROP TABLE", "DROP INDEX"):
    expect(forbidden not in contract_raw, f"no destructive statement {forbidden}")
# No raw email or client-controlled price/grant inputs in the contract.
expect("customer_email" not in contract_raw and "raw_email TEXT" not in contract_raw and "raw_email VARCHAR" not in contract_raw, "no raw email storage field")
expect("'price' =>" not in contract_raw and "['price']" not in contract_raw, "no client-controlled price input")
expect("['grant']" not in contract_raw and "['grants']" not in contract_raw, "no client-controlled grant input")
expect("challenge_hash" in contract_raw and "challenge_used" in contract_raw, "challenge hashed, single-use at rest")
expect("poll_credential_hash" in contract_raw and "hash_only" in contract_raw, "poll credential stored as hash only")

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
expect("********-********-********-XXXX" in fixture_raw, "fixture pins the masked key form")
expect("t***@invalid.example" not in fixture_raw and "t***[AT]invalid.example" not in fixture_raw, "fixture carries no email literal at all")

# ── Journey/journal vectors from the harness ────────────────────────────────

expect(result["journey_states_terminal"] == fixture["journey"]["states_terminal"], "terminal journey states match the pinned state machine")
expect(result["journey_states_agent"] == fixture["journey"]["states_agent"], "agent journey states match the pinned state machine")
expect(result["presenter_states_terminal"] == fixture["journey"]["presenter_states_terminal"], "terminal presenter states match")
expect(result["presenter_states_agent"] == fixture["journey"]["presenter_states_agent"], "agent presenter states match")
expected_counts = dict(fixture["journey"]["journal_vectors"])
expected_counts["journal"] = expected_counts.pop("journal_events")
expect(result["counts"] == expected_counts, "row counts match the pinned journal vectors")
expect(result["sequences"] == {"terminal": 2, "agent": 2}, "sequence ledgers end at 2 after refund")
expect(result["poll_exhausted"] == {"max_polls": 40, "settled": "recovery_only", "poll_count_at_exhaustion": 40}, "bounded poll exhaustion pinned")
expect(result["reveal"] == {"terminal": {"consumed": True}, "agent": {"consumed": True}}, "reveal consumed once per session")
for key in ("receipt_sha256_terminal", "receipt_sha256_agent"):
    expect(re.fullmatch(r"[0-9a-f]{64}", str(result[key])) is not None, f"immutable receipt handle {key} is 64-hex")

positive_checks = result["positive_checks"]
negative_checks = result["negative_checks"]

summary = {
    "schema": "focusa.spec152e.terminal_agent_paid_e2e_validation.v1",
    "atom": "focusa-vbcqu.20.13.57",
    "fixture_sha256": sha256_text(fixture_raw),
    "contract_sha256": sha256_text(contract_raw),
    "harness_sha256": sha256_text(first),
    "positive_checks": positive_checks,
    "negative_checks": negative_checks,
    "journey_states_terminal": len(result["journey_states_terminal"]),
    "journey_states_agent": len(result["journey_states_agent"]),
    "sequences": result["sequences"],
    "receipt_sha256_terminal": result["receipt_sha256_terminal"],
    "receipt_sha256_agent": result["receipt_sha256_agent"],
    "lease_signature_verified": True,
    "terminal_envelope_decrypted_same_canonical_key": True,
    "agent_transcript_masked": True,
    "harness_replay_identical": True,
    "result": "passed",
}
print(json.dumps(summary, sort_keys=True))
