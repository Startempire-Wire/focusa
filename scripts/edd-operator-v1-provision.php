#!/usr/bin/env php
<?php
// 172.02.03 WPUIAI private EDD Operator v1 provisioning command (idempotent).
//
// Reads the canonical Spec 152E product registry and the dedicated Spec 172 Operator v1
// Downloads/price contract, fails closed on any invariant violation, and prints a
// deterministic, redacted provisioning receipt. Running the command any number of times
// produces byte-identical output (idempotent by construction: the plan is derived only
// from the server-owned contracts, ordered canonically, with a stable idempotency key).
//
// Fail-closed invariants enforced here:
//   - exactly three dedicated records; each matches the canonical registry offer
//     (public_code, price_usd, license type/composite SKU, duration, seats, nodes,
//     refund policy, sale status, products, evaluation and future flags);
//   - prices are 69700 / 69700 / 125460 USD minor units and amount_minor equals the
//     human USD string; lifetime duration; whole-order 30-day refund metadata;
//     one operator seat; three shared nodes (operator_shared_v1);
//   - checkout disabled until validation passes (checkout_enabled false everywhere,
//     sale_status approved_not_yet_enabled, download status draft);
//   - dedicated downloads never reuse legacy download IDs or Download 453; the registry
//     never maps Download 453 to any protected offer;
//   - no caller-controlled download, price, License Type, family, feature, limit, node,
//     or commercial right is accepted (contract fields are server-owned only);
//   - the receipt is redacted: no raw email, license key, token, credential, customer
//     row, or card data (the contracts contain none; the receipt asserts this).
//
// No live EDD mutation happens in this repository: the private operator environment
// applies this plan to WPUIAI.com EDD with its own credentials. This command is the
// canonical, safe, idempotent provisioning plan and receipt.
declare(strict_types=1);

$root = dirname(__DIR__);
$registry = require $root . '/docs/contracts/spec152e-edd-product-registry.v1.php';
$dedicated = require $root . '/docs/contracts/spec172-edd-operator-v1-downloads.v1.php';

function spec172_edd_provision_fail(string $message): never
{
    fwrite(STDERR, "PROVISION_FAIL: {$message}\n");
    exit(1);
}

function spec172_edd_provision_assert(bool $condition, string $message): void
{
    if (!$condition) {
        spec172_edd_provision_fail($message);
    }
}

function spec172_edd_provision_assertNoSensitive(array $payload): void
{
    $raw = json_encode($payload, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
    $patterns = [
        '/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/' => 'raw email',
        '/(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+/' => 'stripe key',
        '/focusa_live_[0-9]+_[0-9a-f]+/' => 'synthetic license key',
        '/(?:^|[^A-Za-z0-9])(?:[0-9]{4}[ -]?){3}[0-9]{4}(?:[^0-9]|$)/' => 'card data',
    ];
    foreach ($patterns as $pattern => $label) {
        spec172_edd_provision_assert(preg_match($pattern, $raw) !== 1, "receipt must not expose {$label}");
    }
}

// ── Structural invariants ──────────────────────────────────────────────

spec172_edd_provision_assert(
    ($dedicated['schema'] ?? '') === 'focusa.spec172.edd_operator_v1_downloads.v1',
    'dedicated contract schema',
);
spec172_edd_provision_assert(
    ($registry['schema'] ?? '') === ($dedicated['registry_schema'] ?? ''),
    'registry schema matches dedicated contract reference',
);
spec172_edd_provision_assert(
    ($dedicated['owner'] ?? '') === 'WPUIAI/wpuiai'
    && ($dedicated['provisioner'] ?? '') === 'wpuiai.spec172.edd_operator_v1_provisioner',
    'server-owned owner and provisioner',
);
$records = $dedicated['records'] ?? [];
spec172_edd_provision_assert(count($records) === 3, 'exactly three dedicated records');
spec172_edd_provision_assert(
    ($dedicated['authority']['checkout_enabled'] ?? true) === false,
    'checkout disabled until validation passes',
);
spec172_edd_provision_assert(
    ($dedicated['authority']['checkout_block_reason'] ?? '') === 'awaiting_validation_pass',
    'explicit validation block reason',
);
$legacyIds = array_map('intval', $dedicated['authority']['legacy_download_ids'] ?? []);
$forbiddenImplicit = (int) ($dedicated['authority']['forbidden_implicit_download'] ?? 0);

$offersByCode = [];
foreach ($registry['protected_offers'] as $offer) {
    $offersByCode[(string) $offer['public_code']] = $offer;
}
spec172_edd_provision_assert(count($offersByCode) === 3, 'canonical registry has three protected offers');

$downloadIds = [];
$priceIds = [];
$plan = [];
foreach ($records as $record) {
    $code = (string) ($record['public_code'] ?? '');
    spec172_edd_provision_assert($code !== '', 'record public_code required');
    spec172_edd_provision_assert(isset($offersByCode[$code]), "record {$code} must resolve in the canonical registry");
    $offer = $offersByCode[$code];

    $downloadId = (int) ($record['edd_download_id'] ?? 0);
    spec172_edd_provision_assert($downloadId > 0, "record {$code} requires a positive EDD download id");
    spec172_edd_provision_assert(!in_array($downloadId, $downloadIds, true), "record {$code} download id must be distinct");
    $downloadIds[] = $downloadId;
    spec172_edd_provision_assert(
        !in_array($downloadId, $legacyIds, true) && $downloadId !== $forbiddenImplicit,
        "record {$code} must never reuse legacy or Download 453",
    );

    $priceId = (string) ($record['edd_price_id'] ?? '');
    spec172_edd_provision_assert(preg_match('/^[A-Za-z0-9_]{1,191}$/D', $priceId) === 1, "record {$code} requires a stable price id");
    spec172_edd_provision_assert(!in_array($priceId, $priceIds, true), "record {$code} price id must be distinct");
    $priceIds[] = $priceId;

    $priceUsd = (string) ($record['price_usd'] ?? '');
    $amountMinor = (int) ($record['amount_minor'] ?? 0);
    spec172_edd_provision_assert(
        $priceUsd === (string) ($offer['price_usd'] ?? ''),
        "record {$code} price must match the canonical registry offer",
    );
    spec172_edd_provision_assert(
        preg_match('/^\d{1,10}(\.\d{2})?$/D', $priceUsd) === 1,
        "record {$code} requires a canonical USD price string",
    );
    spec172_edd_provision_assert(
        $amountMinor === (int) round((float) $priceUsd * 100),
        "record {$code} amount_minor must equal price_usd in minor units",
    );
    spec172_edd_provision_assert(
        in_array($amountMinor, [69700, 125460], true),
        "record {$code} must use a Spec 172 server-owned price",
    );

    spec172_edd_provision_assert(
        ($record['license_duration'] ?? '') === 'lifetime'
        && ($offer['license_duration'] ?? '') === 'lifetime'
        && ($record['license_duration'] ?? '') === ($offer['license_duration'] ?? ''),
        "record {$code} must configure lifetime duration matching the offer",
    );
    spec172_edd_provision_assert(
        (int) ($record['operator_seats'] ?? 0) === 1
        && (int) ($offer['operator_seats'] ?? 0) === 1,
        "record {$code} must configure exactly one operator seat",
    );
    spec172_edd_provision_assert(
        (int) ($record['node_limit'] ?? 0) === 3
        && (int) ($offer['node_limit'] ?? 0) === 3
        && ($record['node_set'] ?? '') === 'operator_shared_v1'
        && ($offer['node_set'] ?? '') === 'operator_shared_v1',
        "record {$code} must configure three shared nodes matching the offer",
    );
    spec172_edd_provision_assert(
        ($record['refund_policy'] ?? '') === 'whole_order_30_days'
        && ($offer['refund_policy'] ?? '') === 'whole_order_30_days'
        && (int) ($record['refund_days'] ?? 0) === 30,
        "record {$code} must configure whole-order 30-day refund metadata",
    );
    spec172_edd_provision_assert(
        ($record['sale_status'] ?? '') === 'approved_not_yet_enabled'
        && ($offer['sale_status'] ?? '') === 'approved_not_yet_enabled',
        "record {$code} sale status must match the offer",
    );
    spec172_edd_provision_assert(
        ($record['checkout_enabled'] ?? true) === false
        && ($offer['checkout_enabled'] ?? true) === false,
        "record {$code} checkout must stay disabled",
    );
    spec172_edd_provision_assert(
        ($record['status'] ?? '') === 'draft',
        "record {$code} EDD download must stay draft until validation passes",
    );
    spec172_edd_provision_assert(
        ($record['products'] ?? []) === ($offer['products'] ?? []),
        "record {$code} products must match the offer",
    );
    spec172_edd_provision_assert(
        ($record['evaluation'] ?? true) === false
        && ($record['future_products_included'] ?? true) === false
        && ($record['future_license_types_included'] ?? true) === false,
        "record {$code} must exclude evaluation and future rights",
    );
    spec172_edd_provision_assert(
        ($record['licensing_enabled'] ?? false) === true,
        "record {$code} requires EDD Software Licensing enabled",
    );
    if ($code === 'focusa_uiai_operator_bundle_lifetime_v1') {
        spec172_edd_provision_assert(
            ($record['composite_sku_ref'] ?? '') === ($offer['composite_sku_ref'] ?? ''),
            'bundle composite SKU must match the offer',
        );
        spec172_edd_provision_assert(
            ($record['grants'] ?? []) === ($offer['grants'] ?? [])
            && ($record['grant_composition'] ?? '') === 'exact_union'
            && ($offer['grant_composition'] ?? '') === 'exact_union',
            'bundle must grant the exact two underlying License Types',
        );
        spec172_edd_provision_assert(
            ($record['component_refunds_allowed'] ?? true) === false,
            'bundle refunds are whole-order only',
        );
    } else {
        spec172_edd_provision_assert(
            ($record['license_type_ref'] ?? '') === ($offer['license_type_ref'] ?? ''),
            "record {$code} License Type must match the offer",
        );
    }

    $plan[] = [
        'public_code' => $code,
        'action' => 'create_or_update_dedicated_download',
        'edd_download_id' => $downloadId,
        'edd_price_id' => $priceId,
        'title' => (string) $record['title'],
        'status' => (string) $record['status'],
        'currency' => (string) $record['currency'],
        'price_usd' => $priceUsd,
        'amount_minor' => $amountMinor,
        'licensing_enabled' => true,
        'license_duration' => 'lifetime',
        'license_length' => (int) $record['license_length'],
        'activation_limit' => (int) $record['activation_limit'],
        'operator_seats' => 1,
        'node_limit' => 3,
        'node_set' => 'operator_shared_v1',
        'refund_policy' => 'whole_order_30_days',
        'refund_days' => 30,
        'sale_status' => 'approved_not_yet_enabled',
        'checkout_enabled' => false,
    ];
}

// ── Download 453 and unrelated downloads never grant Operator v1 ───────

foreach ($registry['protected_offers'] as $offer) {
    spec172_edd_provision_assert(
        (int) ($offer['edd_download_id'] ?? 0) !== $forbiddenImplicit,
        'no protected offer may map Download 453',
    );
}
foreach ($registry['current_edd_catalog']['entries'] as $entry) {
    $entryId = (int) $entry['download_id'];
    if ($entryId === $forbiddenImplicit) {
        spec172_edd_provision_assert(
            ($entry['entitlement_disposition'] ?? '') === 'quarantine'
            && ($entry['reason'] ?? '') === 'implicit_focusa_mapping_forbidden',
            'Download 453 must stay quarantined with the explicit forbidden reason',
        );
    }
    spec172_edd_provision_assert(
        !in_array($entryId, $downloadIds, true),
        'dedicated download ids must never collide with catalog entries',
    );
}

// ── Deterministic idempotent receipt ───────────────────────────────────

$canonicalPlan = json_encode($plan, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
$idempotencyKey = hash('sha256', $canonicalPlan);
$receipt = [
    'schema' => 'focusa.spec172.edd_operator_v1_provisioning_receipt.v1',
    'provisioner' => 'wpuiai.spec172.edd_operator_v1_provisioner',
    'source' => [
        'registry' => (string) $registry['schema'],
        'dedicated_downloads' => (string) $dedicated['schema'],
    ],
    'checkout_enabled' => false,
    'checkout_block_reason' => 'awaiting_validation_pass',
    'sale_status' => 'approved_not_yet_enabled',
    'plan' => $plan,
    'counts' => [
        'records' => count($records),
        'checkout_enabled' => 0,
        'assigned_edd_downloads' => count($downloadIds),
        'sum_amount_minor' => array_sum(array_column($records, 'amount_minor')),
    ],
    'idempotency_key' => $idempotencyKey,
    'redacted' => true,
    'excluded' => ['raw_email', 'license_key', 'token', 'credential', 'card_data', 'customer_row', 'caller_supplied_commercial_field'],
    'validation' => 'passed_fail_closed',
];
spec172_edd_provision_assertNoSensitive($receipt);
echo json_encode($receipt, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES) . "\n";
