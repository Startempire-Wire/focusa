#!/usr/bin/env node
/**
 * spec152f_presenter_accessibility_test.mjs
 *
 * Spec 152F.05.02 desktop/TUI presenter parity and accessibility fixtures.
 *
 * Proves that the menubar entitlement presenter and the TUI presenter project
 * the SAME canonical entitlement decision, next action, and recovery action
 * for every entitlement state, and that no disabled control traps the
 * customer away from data or purchase (Spec 152F P5/P6, §6, §11.5, §13; Spec
 * 152E §21 shared presenter vocabulary).
 *
 * What is proven here:
 *
 * 1. PARITY: for every visual entitlement state (active, offline_grace,
 *    expired, revoked, invalid, unactivated) the menubar
 *    `projectEntitlementPosture` resolves the same visual state, presenter
 *    state (frozen Spec 152E vocabulary), next action, and action-guide
 *    action; the TUI `presenter_state_for_entitlement_status` mapping in
 *    crates/focusa-tui/src/activation_presenter.rs uses the identical frozen
 *    mapping.
 * 2. RECOVERY PARITY: both presenters surface the identical recovery policy
 *    sentence ("recovery, export, repair, and uninstall remain available")
 *    and the same always-reachable surface set (navigation, status, account,
 *    read, export, recovery, repair, update, uninstall), including through
 *    the TUI API denial message and the TUI Deck Home view.
 * 3. ACCESSIBILITY: the menubar entitlement component renders an accessible
 *    status region (role=status, aria-live) with an always-reachable action
 *    guide; no `disabled` binding depends on the entitlement posture; the
 *    TUI renders an equivalent action guide; the menubar action map carries
 *    frozen accessibility fixtures and never marks a navigation/display or
 *    recovery/account action as disabled.
 * 4. NO COMPONENT-LOCAL POLICY: neither presenter prices, grants, issues, or
 *    re-decides entitlement; both delegate to the frozen shared presenter
 *    vocabulary and the canonical resolver.
 *
 * Exact verification: node tests/spec152f_presenter_accessibility_test.mjs
 */

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  projectEntitlementPosture,
  ALWAYS_REACHABLE_ACTIONS,
} from '../apps/menubar/src/lib/entitlementPosture.ts';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = resolve(__dirname, '..');

let passed = 0;

function check(condition, message) {
  assert.ok(condition, message);
  passed++;
}

function read(relPath) {
  return readFileSync(resolve(PROJECT_ROOT, relPath), 'utf-8');
}

function loadJson(relPath) {
  return JSON.parse(read(relPath));
}

const MENUBAR_POSTURE = read('apps/menubar/src/lib/entitlementPosture.ts');
const MENUBAR_COMPONENT = read('apps/menubar/src/lib/components/EntitlementPosture.svelte');
const MENUBAR_ACTIVATION = read('apps/menubar/src/lib/activationPresenter.ts');
const TUI_PRESENTER = read('crates/focusa-tui/src/activation_presenter.rs');
const TUI_API = read('crates/focusa-tui/src/api.rs');
const TUI_DECK_HOME = read('crates/focusa-tui/src/views/deck_home.rs');
const ACTION_MAP = loadJson('docs/contracts/spec152f-menubar-action-map.v1.json');

const EXPECTED_ALWAYS_REACHABLE = [
  'navigation',
  'status',
  'account',
  'read',
  'export',
  'recovery',
  'repair',
  'update',
  'uninstall',
];

// ── 1. Parity: menubar posture vs frozen shared presenter mapping ─────────

// [visual state, status, authority, expected action, expected presenter state]
const PARITY = [
  ['active', 'active', { limits: { workflow_runs: 8 } }, 'manage', 'activated'],
  ['offline_grace', 'offline_grace', { offline_grace_until: '2026-08-06T00:00:00Z' }, 'refresh', 'activated'],
  ['expired', 'expired', { recovery_reason: 'lease_expired' }, 'purchase', 'denied'],
  ['revoked', 'recovery_only', { recovery_reason: 'lease_revoked' }, 'purchase', 'recovery_only'],
  ['invalid', 'recovery_only', { recovery_reason: 'signature_invalid' }, 'refresh', 'recovery_only'],
  ['unactivated', 'unactivated', undefined, 'evaluate', 'email_required'],
];

for (const [state, status, authority, action, presenterState] of PARITY) {
  const posture = projectEntitlementPosture({
    status,
    masked_identity: 'o***@example.test',
    authority,
  });
  check(posture.state === state, `${state}: visual state projects as ${state}`);
  check(posture.presenter_state === presenterState, `${state}: frozen presenter state ${presenterState}`);
  check(posture.action === action, `${state}: next action is ${action}`);
  check(posture.action_guide.action === action, `${state}: action guide action is ${action}`);
  check(posture.action_guide.label.length > 0, `${state}: action guide has a label`);
  check(posture.action_guide.explanation.length > 0, `${state}: action guide has an accessible explanation`);
  check(
    JSON.stringify(posture.action_guide.recovery_actions) === JSON.stringify(EXPECTED_ALWAYS_REACHABLE),
    `${state}: action guide keeps the full always-reachable recovery set`,
  );
  check(
    JSON.stringify(posture.always_reachable) === JSON.stringify(EXPECTED_ALWAYS_REACHABLE),
    `${state}: posture carries the full always-reachable set`,
  );
  check(
    /recovery, export, repair, and uninstall/i.test(posture.recovery_policy),
    `${state}: recovery policy keeps data/repair/uninstall reachable`,
  );
}

// The frozen always-reachable fixture is exported by the menubar module.
check(
  JSON.stringify([...ALWAYS_REACHABLE_ACTIONS]) === JSON.stringify(EXPECTED_ALWAYS_REACHABLE),
  'menubar exports the frozen always-reachable action set',
);

// Menubar delegates to the shared frozen vocabulary (Spec 152E §21), never
// re-deciding the presenter state locally.
check(MENUBAR_POSTURE.includes('presenterStateForEntitlementStatus'), 'menubar posture uses the shared status mapping');
check(MENUBAR_POSTURE.includes('presenterNextAction'), 'menubar posture uses the shared next-action table');
check(MENUBAR_POSTURE.includes('allowedActionsFor'), 'menubar posture uses the shared allowed-action table');
check(MENUBAR_POSTURE.includes("'Recovery, export, repair, and uninstall remain available when execution is locked.'"),
  'menubar posture carries the frozen recovery sentence');

// ── 2. TUI parity: identical frozen mapping and recovery surface ──────────

check(TUI_PRESENTER.includes('pub const ALWAYS_REACHABLE'), 'TUI presenter defines the always-reachable fixture');
for (const action of EXPECTED_ALWAYS_REACHABLE) {
  check(TUI_PRESENTER.includes(`"${action}"`), `TUI presenter keeps ${action} always reachable`);
}
check(
  TUI_PRESENTER.includes('"active" | "offline_grace" => TuiPresenterState::Activated'),
  'TUI maps active/offline_grace to activated (frozen mapping)',
);
check(
  TUI_PRESENTER.includes('"expired" | "revoked" => TuiPresenterState::Denied'),
  'TUI maps expired/revoked to denied (frozen mapping)',
);
check(
  TUI_PRESENTER.includes('"recovery_only" => TuiPresenterState::RecoveryOnly'),
  'TUI maps recovery_only to recovery_only (frozen mapping)',
);
check(
  /recovery, export, repair, and uninstall remain available/i.test(TUI_PRESENTER),
  'TUI license posture status line carries the same recovery sentence',
);
check(TUI_PRESENTER.includes('pub fn action_guide('), 'TUI license posture exposes the shared action guide');
check(TUI_PRESENTER.includes('always_reachable=['), 'TUI action guide renders the always-reachable set');

// TUI API denial message surfaces the same recovery allowance (Spec 152E §18).
check(
  /recovery, export, repair, and uninstall remain available/.test(TUI_API),
  'TUI API denial message keeps recovery/export/repair/uninstall reachable',
);

// TUI Deck Home view renders the shared posture and the action guide.
check(TUI_DECK_HOME.includes('Entitlement'), 'TUI Deck Home renders the entitlement posture');
check(TUI_DECK_HOME.includes('status_line()'), 'TUI Deck Home renders the posture status line');
check(TUI_DECK_HOME.includes('action_guide()'), 'TUI Deck Home renders the shared action guide');
check(TUI_DECK_HOME.includes('unavailable (no signed authority snapshot)'),
  'TUI Deck Home fails closed instead of inventing a posture');

// ── 3. Accessibility: no disabled control traps the customer ──────────────

// Menubar component: accessible status region; the only disabled binding is
// the busy state of the Refresh control, never the entitlement posture.
check(MENUBAR_COMPONENT.includes('role="status"'), 'menubar entitlement component is a status region');
check(MENUBAR_COMPONENT.includes('aria-live="polite"'), 'menubar entitlement component announces posture changes');
check(MENUBAR_COMPONENT.includes('aria-label="Entitlement posture"'), 'menubar entitlement component has a label');
check(!MENUBAR_COMPONENT.includes('disabled={posture'), 'menubar never disables a control based on entitlement posture');
check(MENUBAR_COMPONENT.includes('posture.action_guide.label'), 'menubar renders the next-action label');
check(MENUBAR_COMPONENT.includes('posture.always_reachable.join'), 'menubar renders the always-reachable set');

// Action map: navigation/display and recovery/account actions are never
// disabled by an entitlement decision; canonical operations preserve recovery.
for (const action of ACTION_MAP.actions) {
  check(!('disabled' in action), `${action.baseline_id}: action map never marks a control disabled`);
  if (action.action_class === 'navigation_display') {
    check(
      ACTION_MAP.action_classes.navigation_display.policy === 'no_entitlement_check_required',
      `${action.baseline_id}: navigation/display requires no entitlement check`,
    );
  }
  if (action.action_class === 'recovery_account') {
    check(
      ACTION_MAP.action_classes.recovery_account.policy === 'always_available_subject_to_security',
      `${action.baseline_id}: recovery/account stays reachable subject to security`,
    );
  }
  if (action.action_class === 'canonical_operation') {
    const must = ACTION_MAP.action_classes.canonical_operation.presenter_must;
    check(must.includes('preserve_recovery_paths'), `${action.baseline_id}: canonical operation preserves recovery paths`);
    check(must.includes('forward_daemon_error_transparently'), `${action.baseline_id}: canonical operation forwards daemon denial`);
  }
}

// Frozen accessibility fixtures on the action map.
check(
  JSON.stringify(ACTION_MAP.accessibility_fixtures.always_reachable) === JSON.stringify(EXPECTED_ALWAYS_REACHABLE),
  'action map accessibility fixture matches the frozen always-reachable set',
);
const fixtureRules = ACTION_MAP.accessibility_fixtures.rules.join(' ').toLowerCase();
check(fixtureRules.includes('no disabled control traps the customer away from data or purchase'),
  'action map fixture: no disabled control traps the customer');
check(fixtureRules.includes('accessible explanation') && fixtureRules.includes('evaluation/purchase/recovery'),
  'action map fixture: denied value actions carry explanation and Evaluation/purchase/recovery action');
check(fixtureRules.includes('never disabled by an entitlement decision'),
  'action map fixture: always-reachable surfaces are never disabled');

// ── 4. No component-local policy anywhere in the presenters ───────────────

const FORBIDDEN_POLICY_MARKERS = [
  'price',
  'pricing',
  'sku',
  'mint_grants',
  'issue_lease',
  'self_eval',
  'evaluation_receipt',
  'installer_grace',
  'EDD_SL_KEY',
  'write_license',
  'grant_reason',
  'caller_feature',
  'caller_bucket',
];
for (const marker of FORBIDDEN_POLICY_MARKERS) {
  check(!MENUBAR_POSTURE.includes(marker), `menubar posture has no component-local policy marker ${marker}`);
  check(!TUI_PRESENTER.includes(marker), `TUI presenter has no component-local policy marker ${marker}`);
}
// Neither presenter invents identity, payment, or lease authority locally.
for (const surface of [MENUBAR_POSTURE, MENUBAR_ACTIVATION, TUI_PRESENTER]) {
  check(!surface.includes('raw_email'), 'presenters carry no raw email field');
  check(!surface.includes('full_license_key'), 'presenters carry no full license key');
  check(!surface.includes('poll_credential'), 'presenters carry no poll credential');
}

// ── Summary ───────────────────────────────────────────────────────────────

console.log(`spec152f presenter parity and accessibility: PASS (${passed} assertions)`);
console.log('  parity: menubar and TUI project the same decision and recovery action');
console.log('  accessibility: no disabled control traps the customer away from data or purchase');
process.exit(0);
