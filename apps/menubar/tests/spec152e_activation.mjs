// Spec 152E.05.06 menubar first-run/entitlement presenter contract.
//
// Binds the menubar's first-run/entitlement posture to the frozen Spec 152E
// presenter vocabulary: the same shared activation states, next actions,
// allowed actions, masked identity, checkout/verify links, terminal delivery,
// node management, denial/recovery, and resume handles that the TUI, the
// daemon REST license routes, and lifecycle receipts expose for one canonical
// registration. The menubar renders only; the shared reducer decides.
//
// Exact verification: node apps/menubar/tests/spec152e_activation.mjs

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  PRESENTER_STATES,
  TERMINAL_STATES,
  allowedActionsFor,
  denialRecovery,
  isMaskedIdentity,
  presenterNextAction,
  presenterStateForEntitlementStatus,
  projectActivationStatus,
  projectLicenseStatus,
  safeAuthorityLink,
} from '../src/lib/activationPresenter.ts';
import { projectEntitlementPosture } from '../src/lib/entitlementPosture.ts';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repo = path.resolve(root, '..', '..');
const INTERNAL = JSON.parse(
  fs.readFileSync(
    path.join(repo, 'docs/contracts/spec152e-activation-internal.v1.json'),
    'utf8',
  ),
);
const ERRORS = JSON.parse(
  fs.readFileSync(
    path.join(repo, 'docs/contracts/spec152e-activation-errors.v1.json'),
    'utf8',
  ),
);
const WIZARD = fs.readFileSync(
  path.join(root, 'src/lib/components/FirstRunWizard.svelte'),
  'utf8',
);

let positive = 0;
let negative = 0;
function expect(condition, message, isNegative = false) {
  if (isNegative) negative += 1;
  else positive += 1;
  if (!condition) throw new Error(message);
}

// ── Frozen presenter vocabulary ──────────────────────────────────────────

const frozenStates = INTERNAL.presenter_states;
expect(
  JSON.stringify(PRESENTER_STATES) === JSON.stringify(frozenStates),
  'menubar presenter states are byte-exact with the frozen contract',
);
expect(
  PRESENTER_STATES.length === 10,
  'frozen machine has exactly 10 presenter states',
);

const terminal = new Set(INTERNAL.polling.terminal_states);
expect(
  JSON.stringify([...TERMINAL_STATES].sort()) ===
    JSON.stringify([...terminal].sort()),
  'terminal presenter states match the frozen polling contract',
);

// ── Next-action table is the frozen shared table ─────────────────────────

const nextActions = Object.fromEntries(
  PRESENTER_STATES.map((s) => [s, presenterNextAction(s)]),
);
expect(
  nextActions.email_required === 'provide_email',
  'email_required next action',
);
expect(
  nextActions.email_verification_pending === 'verify_email',
  'verification pending next action',
);
expect(
  nextActions.selection_required === 'select_offer',
  'selection next action',
);
expect(
  nextActions.checkout_required === 'open_checkout',
  'checkout next action',
);
expect(
  nextActions.payment_pending === 'poll_after_retry_after',
  'payment next action',
);
expect(
  nextActions.license_delivery_ready === 'deliver_license',
  'delivery next action',
);
expect(nextActions.activated === 'activated', 'activated next action');
expect(
  nextActions.denied === 'activate_or_manage_entitlement',
  'denied next action',
);
expect(nextActions.recovery_only === 'recovery', 'recovery next action');

// ── Equivalent allowed actions across one canonical registration ─────────

expect(
  JSON.stringify(allowedActionsFor('checkout_required')) ===
    JSON.stringify(['open_checkout']),
  'checkout_required exposes open_checkout',
);
expect(
  JSON.stringify(allowedActionsFor('payment_pending').sort()) ===
    JSON.stringify(['open_checkout', 'poll'].sort()),
  'payment_pending exposes poll + open_checkout',
);
expect(
  allowedActionsFor('activated').includes('manage_nodes') &&
    allowedActionsFor('activated').includes('refresh_lease') &&
    allowedActionsFor('activated').includes('resume'),
  'activated exposes node management, lease refresh, and resume',
);
for (const action of [
  'recovery',
  'repair',
  'export',
  'uninstall',
  'manage_nodes',
  'manage_account',
]) {
  expect(
    allowedActionsFor('recovery_only').includes(action),
    `recovery_only exposes ${action}`,
  );
}
expect(
  JSON.stringify(allowedActionsFor('denied').sort()) ===
    JSON.stringify(['activate_or_manage_entitlement', 'recovery'].sort()),
  'denied exposes activation-or-manage and recovery',
);

// ── Masked identity and safe links fail closed ───────────────────────────

expect(
  isMaskedIdentity('c***@example.com') === true,
  'masked identity accepted',
);
expect(
  isMaskedIdentity('raw@example.com') === false,
  'unmasked identity rejected',
);
expect(isMaskedIdentity('c***@') === false, 'empty domain rejected');
expect(isMaskedIdentity('c@example.com') === false, 'no star rejected');
expect(
  safeAuthorityLink('https://install.focusa.dev/pay/opaque') !== null,
  'https link accepted',
);
expect(
  safeAuthorityLink('http://evil.example.test/pay') === null,
  'http link rejected',
);
expect(
  safeAuthorityLink('https://user:pass@evil.example.test/pay') === null,
  'credential-bearing link rejected',
);
expect(safeAuthorityLink('not a url') === null, 'garbage link rejected');

// ── Deterministic projection from the daemon activation surface ───────────

const view = projectActivationStatus({
  registrations: [
    {
      registration_id: 'registration-0001',
      state: 'payment_pending',
      masked_email: 'c***@example.com',
      safe_url: 'https://install.focusa.dev/pay/opaque-token',
      retry_posture: 'safe_retry',
    },
    {
      registration_id: 'registration-0002',
      state: 'recovery_only',
      masked_email: 'o***@example.com',
    },
  ],
});
expect(view !== null, 'first registration projects');
expect(view.state === 'payment_pending', 'projected state');
expect(view.next_action === 'poll_after_retry_after', 'projected next action');
expect(
  view.actions.includes('poll') && view.actions.includes('open_checkout'),
  'projected actions',
);
expect(view.masked_email === 'c***@example.com', 'projected masked email');
expect(
  view.safe_url === 'https://install.focusa.dev/pay/opaque-token',
  'projected safe link',
);
expect(
  view.resume_handle === 'registration-0001',
  'resume handle for non-terminal registration',
);

const terminalView = projectActivationStatus({
  registrations: [
    {
      registration_id: 'registration-0009',
      state: 'recovery_only',
      masked_email: 'o***@example.com',
    },
  ],
});
expect(terminalView.terminal === true, 'terminal projection');
expect(
  terminalView.resume_handle === undefined,
  'no resume handle on terminal states',
);
expect(
  denialRecovery(terminalView).recovery_only === true,
  'recovery-only denial rendering',
);
expect(
  denialRecovery(terminalView).recovery_actions.includes('repair') &&
    denialRecovery(terminalView).recovery_actions.includes('uninstall'),
  'recovery actions rendered',
);

// Unknown states fail closed instead of inventing a grant.
expect(
  projectActivationStatus({
    registrations: [{ registration_id: 'x', state: 'granted_now' }],
  }) === null,
  'unknown state fails closed',
);
// Unmasked emails fail closed (never rendered).
expect(
  projectActivationStatus({
    registrations: [
      {
        registration_id: 'x',
        state: 'payment_pending',
        masked_email: 'raw@example.com',
      },
    ],
  }).masked_email === undefined,
  'unmasked email never rendered',
);
// Deterministic: same payload → same view.
expect(
  JSON.stringify(
    projectActivationStatus({
      registrations: [
        { registration_id: 'registration-0001', state: 'checkout_required' },
      ],
    }),
  ) ===
    JSON.stringify(
      projectActivationStatus({
        registrations: [
          { registration_id: 'registration-0001', state: 'checkout_required' },
        ],
      }),
    ),
  'projection is deterministic',
);

// ── License status maps onto the same shared vocabulary ───────────────────

expect(
  projectLicenseStatus({ status: 'active' }).presenter_state === 'activated',
  'active → activated',
);
expect(
  projectLicenseStatus({ status: 'offline_grace' }).presenter_state ===
    'activated',
  'offline_grace → activated',
);
expect(
  projectLicenseStatus({ status: 'recovery_only' }).presenter_state ===
    'recovery_only',
  'recovery_only → recovery_only',
);
expect(
  projectLicenseStatus({ status: 'expired' }).presenter_state === 'denied',
  'expired → denied',
);
expect(
  projectLicenseStatus({ status: 'unactivated' }).presenter_state ===
    'email_required',
  'unactivated → email_required',
);
expect(
  projectLicenseStatus({
    status: 'active',
    masked_identity: 'o***@example.com',
  }).masked_identity === 'o***@example.com',
  'masked identity projected',
);
expect(
  projectLicenseStatus({ status: 'active', masked_identity: 'o@example.com' })
    .masked_identity === undefined,
  'unmasked identity dropped',
);

const posture = projectEntitlementPosture({
  status: 'active',
  masked_identity: 'o***@example.test',
  authority: { limits: { workflow_runs: 8 } },
});
expect(
  posture.presenter_state === 'activated',
  'entitlement posture carries shared presenter state',
);
expect(
  posture.next_action === 'activated',
  'entitlement posture carries shared next action',
);
expect(
  posture.allowed_actions.includes('manage_nodes'),
  'entitlement posture carries shared allowed actions',
);

// ── Menubar surface is wired to the shared presenter (no duplicate logic) ─

expect(
  WIZARD.includes("from '$lib/activationPresenter'"),
  'wizard imports the shared presenter',
);
expect(
  WIZARD.includes('projectActivationStatus'),
  'wizard projects the shared activation view',
);
expect(
  WIZARD.includes('/v1/activation/status'),
  'wizard reads the daemon activation surface',
);
expect(WIZARD.includes('resume_handle'), 'wizard renders the resume handle');
expect(
  WIZARD.includes('Recovery only:'),
  'wizard renders recovery-only posture',
);
expect(WIZARD.includes('Allowed actions:'), 'wizard renders allowed actions');
// The menubar never invents a verification code, consent, payment, or license.
expect(
  !/invented|mock.*verification|local.*license/i.test(WIZARD),
  'wizard never invents authority decisions',
);

// ── Secrets and raw emails are absent from the presenter artifacts ───────

const presenterSource = fs.readFileSync(
  path.join(root, 'src/lib/activationPresenter.ts'),
  'utf8',
);
for (const forbidden of [
  'access_token',
  'refresh_token',
  'poll_credential',
  'full_license_key',
  'card_pan',
  'card_cvc',
]) {
  expect(
    !presenterSource.includes(forbidden),
    `activationPresenter.ts has no ${forbidden} field`,
    true,
  );
}
expect(
  !presenterSource.includes('@example.com'),
  'no example emails baked into the presenter',
  true,
);

console.log(
  `menubar spec152e activation presenter tests passed (positive=${positive} negative=${negative})`,
);
