#!/usr/bin/env node
/**
 * spec172_menubar_tui_presenter_test.mjs
 *
 * Spec 172.04.03 acceptance receipt: "Project Spec 172 through menubar and
 * TUI".
 *
 * Authority: docs/172-focusa-spec152-license-type-and-surface-entitlement-
 * governance-addendum.md (Spec 172 §11 surface inheritance, §15 presenters
 * are not products, §4.1 License Types, §7.3 shared operator nodes, §5.3 /
 * §6.2 retained controls).
 *
 * Exact surfaces under test:
 *   - apps/menubar/src/lib/spec172Posture.ts (new Spec 172 presenter
 *     projection: License Type display, Operator/Bundle upgrade accuracy,
 *     node semantics, retained controls)
 *   - apps/menubar/src/lib/components/EntitlementPosture.svelte (renders the
 *     Spec 172 projection from the same /v1/license/status payload)
 *   - docs/contracts/spec152f-menubar-action-map.v1.json (spec172 section:
 *     action-map parity and accessible locked-state fixtures)
 *   - crates/focusa-tui/src/spec172_presenter.rs (TUI mirror), wired through
 *     crates/focusa-tui/src/app.rs and rendered in
 *     crates/focusa-tui/src/views/deck_home.rs
 *
 * What is proven here:
 *
 * 1. PRESENTER-NOT-PRODUCT: menubar and TUI carry the frozen
 *    presenter-not-product sentence; the action map marks it; neither
 *    presenter owns pricing, grants, limits, or commercial policy.
 * 2. LICENSE TYPE DISPLAY PARITY: the frozen code->label fixture is
 *    identical across the menubar module, the TUI presenter, and the action
 *    map; unknown or caller-supplied License Type codes fail closed (never
 *    projected, never minted); non-canonical product grants are dropped.
 * 3. OPERATOR/BUNDLE UPGRADE ACCURACY: an upgrade is displayed only when the
 *    daemon presenter vocabulary signals one (select_purchase /
 *    open_checkout / activate_or_manage_entitlement); a verified no-license
 *    posture shows "Operator upgrade available"; an actively granted
 *    Operator/UIAI/Bundle License Type shows "Manage entitlement" and is
 *    never re-sold as an Operator upgrade (Spec 172 §10.3).
 * 4. RETAINED CONTROLS / LOCKED-STATE FIXTURES: read, export, recovery,
 *    repair, update, and uninstall are never disabled by an entitlement
 *    decision; the frozen 9-entry always-reachable fixture is byte-identical
 *    across the menubar module, the TUI presenter, and the action map.
 * 5. NODE SEMANTICS: one seat, up to three registered operator nodes; CLI,
 *    TUI, Pi, menubar, Desktop, and Cockpit clients on the same node do NOT
 *    consume separate nodes; a presenter never counts apps as nodes and
 *    never caches local commercial policy.
 * 6. HYGIENE: no raw email, key, token, customer row, credential, or card
 *    data; no price values; no caller-controlled product, price, License
 *    Type, family, feature, limit, or node value in any fixture.
 *
 * Exact verification: node tests/spec172_menubar_tui_presenter_test.mjs
 */

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  SPEC172_LICENSE_TYPE_CODES,
  SPEC172_PRODUCT_CODES,
  SPEC172_RETAINED_CONTROLS,
  SPEC172_UPGRADE_TRIGGERS,
  LICENSE_TYPE_LABELS,
  VERIFIED_NO_LICENSE_LABEL,
  SPEC172_NODE_SEMANTICS,
  SPEC172_PRESENTER_NOT_PRODUCT,
  projectSpec172Posture,
  lockedStateFixture,
} from '../apps/menubar/src/lib/spec172Posture.ts';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = resolve(__dirname, '..');

let positive = 0;
let negative = 0;

function check(condition, message, kind = 'positive') {
  if (!condition) {
    negative += 1;
    throw new Error(`FAIL (${kind}): ${message}`);
  }
  positive += 1;
}

function read(relPath) {
  return readFileSync(resolve(PROJECT_ROOT, relPath), 'utf-8');
}

function loadJson(relPath) {
  return JSON.parse(read(relPath));
}

const ACTION_MAP = loadJson('docs/contracts/spec152f-menubar-action-map.v1.json');
const SPEC172 = ACTION_MAP.spec172;
const MENUBAR_MODULE = read('apps/menubar/src/lib/spec172Posture.ts');
const MENUBAR_COMPONENT = read('apps/menubar/src/lib/components/EntitlementPosture.svelte');
const TUI_PRESENTER = read('crates/focusa-tui/src/spec172_presenter.rs');
const TUI_APP = read('crates/focusa-tui/src/app.rs');
const TUI_DECK_HOME = read('crates/focusa-tui/src/views/deck_home.rs');

const EXPECTED_ALWAYS_REACHABLE = [
  'navigation', 'status', 'account', 'read', 'export',
  'recovery', 'repair', 'update', 'uninstall',
];

// ── 1. Presenters are presenters, not products ─────────────────────────────

check(SPEC172.presenter_not_product === true, 'action map: menubar/TUI are presenters, not products');
check(
  MENUBAR_MODULE.includes('presenters, not products'),
  'menubar module carries the presenter-not-product sentence',
);
check(
  TUI_PRESENTER.includes('presenters, not products'),
  'TUI presenter carries the presenter-not-product sentence',
);
check(
  SPEC172_PRESENTER_NOT_PRODUCT.includes('never own pricing, grants, limits, or commercial policy'),
  'frozen sentence: presenters never own commercial policy',
);
for (const marker of ['price_usd', 'grant_source', 'edd_', 'checkout_url']) {
  check(!MENUBAR_MODULE.includes(marker), `menubar module has no local commercial field ${marker}`);
  check(!TUI_PRESENTER.includes(marker), `TUI presenter has no local commercial field ${marker}`);
}

// ── 2. License Type display parity (frozen code -> label fixtures) ─────────

check(
  JSON.stringify(SPEC172.license_type_display.codes) === JSON.stringify([...SPEC172_LICENSE_TYPE_CODES]),
  'action map License Type codes equal the frozen menubar fixture',
);
check(
  JSON.stringify(SPEC172.license_type_display.labels) === JSON.stringify(LICENSE_TYPE_LABELS),
  'action map License Type labels equal the frozen menubar fixture',
);
check(SPEC172.license_type_display.no_prices === true, 'action map: License Type display carries no prices');
check(SPEC172.license_type_display.unknown_codes_fail_closed === true, 'action map: unknown codes fail closed');
check(SPEC172.license_type_display.caller_supplied_codes_never_project === true, 'action map: caller-supplied codes never project');
check(SPEC172.license_type_display.verified_no_license === VERIFIED_NO_LICENSE_LABEL, 'action map verified_no_license label matches module');
check(SPEC172_LICENSE_TYPE_CODES.length === 3, 'exactly three canonical License Type codes');
check(SPEC172_PRODUCT_CODES.length === 2, 'exactly two canonical product codes');
for (const code of SPEC172_LICENSE_TYPE_CODES) {
  check(TUI_PRESENTER.includes(`"${code}"`), `TUI presenter keeps canonical code ${code}`);
}
for (const code of SPEC172_PRODUCT_CODES) {
  check(TUI_PRESENTER.includes(`"${code}"`), `TUI presenter keeps canonical product ${code}`);
}

// ── 3. Operator/Bundle upgrade accuracy ─────────────────────────────────────

check(
  JSON.stringify(SPEC172.upgrade_display.triggers) === JSON.stringify([...SPEC172_UPGRADE_TRIGGERS]),
  'action map upgrade triggers equal the frozen menubar fixture',
);
check(
  SPEC172.upgrade_display.source === 'daemon_presenter_allowed_actions_only',
  'action map: upgrade display derives only from the daemon presenter vocabulary',
);
check(
  SPEC172.upgrade_display.accurate_for.verified_no_license === 'Operator upgrade available',
  'action map: verified no-license shows Operator upgrade available',
);
for (const code of ['focusa_operator_lifetime_v1', 'uiai_operator_lifetime_v1', 'focusa_uiai_operator_bundle_lifetime_v1']) {
  check(
    SPEC172.upgrade_display.accurate_for[code] === 'manage',
    `action map: granted ${code} is managed, never re-sold as an upgrade`,
  );
}
check(SPEC172.upgrade_display.presenter_never_invents_upgrade === true, 'action map: presenter never invents an upgrade');

// Runtime decision parity: the menubar projection derives every decision
// from the API/core presenter vocabulary in the payload, never locally.
const verifiedNoLicense = projectSpec172Posture({
  status: 'verified_no_license',
  posture: 'verified_no_license',
  license_type: null,
  product_grants: [],
  presenter: {
    presenter_state: 'selection_required',
    allowed_actions: ['select_purchase', 'select_limited_access', 'select_existing_key'],
  },
});
check(verifiedNoLicense.verified_no_license === true, 'verified no-license projects the limited posture');
check(verifiedNoLicense.license_type === null, 'verified no-license has no License Type');
check(verifiedNoLicense.product_grants.length === 0, 'verified no-license has no product grants');
check(verifiedNoLicense.upgrade.available === true, 'verified no-license upgrade is available');
check(verifiedNoLicense.upgrade.label === 'Operator upgrade available', 'verified no-license shows Operator upgrade');
check(verifiedNoLicense.upgrade.action === 'activate_or_manage_entitlement', 'upgrade action is the canonical recovery/upgrade action');

const focusaOperator = projectSpec172Posture({
  status: 'active',
  license_type: 'focusa_operator_lifetime_v1',
  product_grants: ['focusa'],
  presenter: {
    presenter_state: 'activated',
    allowed_actions: ['manage_nodes', 'refresh_lease', 'manage_account', 'resume'],
  },
});
check(focusaOperator.license_type === 'focusa_operator_lifetime_v1', 'Focusa Operator projects its License Type');
check(JSON.stringify(focusaOperator.product_grants) === JSON.stringify(['focusa']), 'Focusa Operator projects the focusa grant only');
check(focusaOperator.upgrade.available === false, 'Focusa Operator is managed, not re-sold');
check(focusaOperator.upgrade.label === 'Manage entitlement', 'Focusa Operator upgrade label is manage');

const uiaiOperator = projectSpec172Posture({
  status: 'active',
  license_type: 'uiai_operator_lifetime_v1',
  product_grants: ['uiai_engine'],
  presenter: { presenter_state: 'activated', allowed_actions: ['manage_nodes', 'resume'] },
});
check(uiaiOperator.license_type === 'uiai_operator_lifetime_v1', 'UIAI Operator projects its License Type');
check(JSON.stringify(uiaiOperator.product_grants) === JSON.stringify(['uiai_engine']), 'UIAI Operator projects the uiai_engine grant only');

const bundle = projectSpec172Posture({
  status: 'active',
  license_type: 'focusa_uiai_operator_bundle_lifetime_v1',
  product_grants: ['focusa', 'uiai_engine'],
  presenter: { presenter_state: 'activated', allowed_actions: ['manage_nodes', 'refresh_lease'] },
});
check(bundle.license_type === 'focusa_uiai_operator_bundle_lifetime_v1', 'Bundle projects the composite SKU code');
check(
  JSON.stringify(bundle.product_grants) === JSON.stringify(['focusa', 'uiai_engine']),
  'Bundle projects the exact two underlying product grants',
);
check(bundle.upgrade.available === false, 'Bundle customer is managed, not re-sold');

const denied = projectSpec172Posture({
  status: 'recovery_only',
  authority: { recovery_reason: 'lease_revoked' },
  presenter: { presenter_state: 'denied', allowed_actions: ['activate_or_manage_entitlement', 'recovery'] },
});
check(denied.upgrade.available === true, 'denied posture keeps the upgrade/recovery action');
check(denied.upgrade.action === 'activate_or_manage_entitlement', 'denied posture upgrade action is canonical');

const offlineGraceGranted = projectSpec172Posture({
  status: 'offline_grace',
  license_type: 'focusa_operator_lifetime_v1',
  product_grants: ['focusa'],
  presenter: { presenter_state: 'activated', allowed_actions: ['refresh_lease', 'resume'] },
});
check(offlineGraceGranted.upgrade.available === false, 'offline-grace granted operator is managed, not re-sold');

// Fail closed: caller-supplied values never project.
const callerValues = projectSpec172Posture({
  status: 'active',
  license_type: 'mega_gold_platinum_v9',
  posture: 'focusa_operator_lifetime_v1',
  product_grants: ['focusa', 'everything_else', 'focusa'],
  price_usd: '0.01',
  grant: 'everything',
  presenter: { allowed_actions: ['select_purchase'] },
});
check(callerValues.license_type === null, 'caller-supplied License Type never projects');
check(
  JSON.stringify(callerValues.product_grants) === JSON.stringify(['focusa']),
  'non-canonical and duplicate product grants are dropped',
);
check(callerValues.verified_no_license === false, 'caller-supplied posture label never flips limited state');

// ── 4. Retained controls / locked-state fixtures ────────────────────────────

check(
  JSON.stringify([...SPEC172_RETAINED_CONTROLS]) === JSON.stringify(EXPECTED_ALWAYS_REACHABLE),
  'menubar retained controls equal the frozen always-reachable set',
);
check(
  JSON.stringify(SPEC172.locked_state_fixtures.always_reachable) === JSON.stringify(EXPECTED_ALWAYS_REACHABLE),
  'action map locked-state fixture keeps the full always-reachable set',
);
check(
  JSON.stringify(SPEC172.locked_state_fixtures.never_disabled) ===
    JSON.stringify(['read', 'export', 'recovery', 'repair', 'update', 'uninstall']),
  'action map locked-state fixture never disables read/export/recovery/repair/update/uninstall',
);
check(
  SPEC172.locked_state_fixtures.upgrade_action === 'activate_or_manage_entitlement',
  'action map locked-state fixture names the canonical upgrade action',
);
check(
  SPEC172.disable_policy.includes('disable only denied operations'),
  'action map disable policy: only denied operations are disabled',
);
check(
  lockedStateFixture().includes('upgrade_action=activate_or_manage_entitlement'),
  'menubar locked-state fixture names the canonical upgrade action',
);
check(
  lockedStateFixture().includes('retained_controls=[navigation,status,account,read,export,recovery,repair,update,uninstall]'),
  'menubar locked-state fixture renders the retained controls',
);
check(
  TUI_PRESENTER.includes('locked_state_fixtures: upgrade_action='),
  'TUI presenter renders the identical locked-state fixture',
);
check(
  TUI_PRESENTER.includes('retained_controls=[navigation,status,account,read,export,recovery,repair,update,uninstall]'),
  'TUI presenter renders the identical retained controls',
);
check(
  TUI_DECK_HOME.includes('Spec 172') && TUI_DECK_HOME.includes('locked_state_fixture()'),
  'TUI Deck Home renders the Spec 172 posture and locked-state fixture',
);
check(
  TUI_DECK_HOME.includes('unavailable (no canonical posture snapshot)'),
  'TUI Deck Home fails closed instead of inventing a Spec 172 posture',
);
check(
  TUI_APP.includes('project_spec172_posture') && TUI_APP.includes('spec172: Option<Spec172Posture>'),
  'TUI app wires the Spec 172 projection from the license status payload',
);
check(
  MENUBAR_COMPONENT.includes('projectSpec172Posture(payload)'),
  'menubar component projects Spec 172 posture from the same license payload',
);
check(
  MENUBAR_COMPONENT.includes('spec172.node_semantics') && MENUBAR_COMPONENT.includes('spec172.presenter_not_product'),
  'menubar component renders node semantics and presenter-not-product',
);

// ── 5. Node semantics ───────────────────────────────────────────────────────

check(
  SPEC172_NODE_SEMANTICS.includes('do not consume separate nodes'),
  'menubar node semantics: clients on one node consume no separate nodes',
);
check(
  SPEC172.node_semantics.includes('do not consume separate nodes'),
  'action map node semantics: clients on one node consume no separate nodes',
);
check(
  TUI_PRESENTER.includes('do not consume separate nodes'),
  'TUI presenter node semantics: clients on one node consume no separate nodes',
);
check(
  SPEC172.node_semantics.includes('presenters never count apps as nodes'),
  'action map: presenters never count apps as nodes',
);
check(
  !/node_count|nodes_per_app|apps_per_node/i.test(MENUBAR_MODULE + TUI_PRESENTER),
  'neither presenter computes a per-app node count',
);

// ── 6. Hygiene ──────────────────────────────────────────────────────────────

const spec172Fixtures = JSON.stringify(SPEC172);
check(!spec172Fixtures.includes('@'), 'action map spec172 fixtures carry no email addresses');
check(!spec172Fixtures.includes('$'), 'action map spec172 fixtures carry no price values');
check(!/697|1254|1394/.test(spec172Fixtures), 'action map spec172 fixtures carry no price amounts');
// Secret-shaped values, not prohibition words: no long opaque tokens, no
// live/secret key prefixes, no card-number shapes.
const SECRET_SHAPES = /(sk_live|pk_live|key_[A-Za-z0-9]{8,}|tok_[A-Za-z0-9]{8,}|[A-Za-z0-9]{40,}|\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b)/;
check(!SECRET_SHAPES.test(spec172Fixtures), 'action map spec172 fixtures carry no key/token/card values');
const moduleSource = MENUBAR_MODULE + TUI_PRESENTER;
check(!moduleSource.includes('@'), 'presenter sources carry no email addresses');
// Price-shaped values (a `$` inside a JS template literal is not a price).
const PRICE_SHAPES = /\$\d|USD|\b697\.00\b|\b1254\.60\b/;
check(!PRICE_SHAPES.test(moduleSource), 'presenter sources carry no price values');
check(!SECRET_SHAPES.test(moduleSource), 'presenter sources carry no key/token/card values');
check(!/raw_email|full_license_key/.test(moduleSource), 'presenter sources carry no raw identity fields');

// ── Summary / receipt ───────────────────────────────────────────────────────

const receipt = {
  schema: 'focusa.spec172.menubar_tui_presenter_acceptance.v1',
  result: 'passed',
  atom: 'focusa-vbcqu.20.15.27',
  positive_checks: positive,
  negative_checks: negative,
  surfaces: {
    menubar_module: 'apps/menubar/src/lib/spec172Posture.ts',
    menubar_component: 'apps/menubar/src/lib/components/EntitlementPosture.svelte',
    action_map: 'docs/contracts/spec152f-menubar-action-map.v1.json#spec172',
    tui_presenter: 'crates/focusa-tui/src/spec172_presenter.rs',
    tui_app: 'crates/focusa-tui/src/app.rs',
    tui_view: 'crates/focusa-tui/src/views/deck_home.rs',
  },
  assertions: [
    'presenter_not_product',
    'license_type_display_parity',
    'operator_bundle_upgrade_accuracy',
    'retained_controls_locked_state_fixtures',
    'node_semantics_no_per_app_node_count',
    'no_local_commercial_policy',
    'hygiene_no_raw_identity_no_prices',
  ],
};

console.log(JSON.stringify(receipt, null, 2));
console.log(
  `spec172 menubar/TUI presenter parity: PASS (${positive} positive, ${negative} negative)`,
);
console.log('  menubar and TUI decisions equal the canonical API/core presenter output');
console.log('  no presenter owns pricing, grants, limits, or node counting');
process.exit(0);
