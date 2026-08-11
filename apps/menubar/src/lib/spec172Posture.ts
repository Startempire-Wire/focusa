// Menubar Spec 172 presenter projection (docs/172-focusa-spec152-license-type-
// and-surface-entitlement-governance-addendum.md §11, §15).
//
// Menubar and TUI are presenters, not products (Spec 172 §15): they project
// the canonical operation decision from the daemon `GET /v1/license/status`
// payload and never own pricing, grants, limits, License Type selection, or
// commercial policy. This module renders:
//
//   - the canonical License Type display (Operator / Bundle / verified
//     no-license limited posture) from FROZEN code->label fixtures only;
//   - an accurate upgrade display derived ONLY from the daemon presenter's
//     own allowed-actions vocabulary — a presenter never invents an upgrade;
//   - the frozen node semantics (Spec 172 §7.3): CLI, TUI, Pi, menubar,
//     Focusa Desktop, and other approved clients on the same node do NOT
//     consume separate nodes;
//   - the frozen retained/always-reachable controls (Spec 172 §5.3, §6.2):
//     navigation, status, account, read, export, recovery, repair, update,
//     and uninstall are never disabled by an entitlement decision.
//
// Fail-closed rules:
//   - an unknown or caller-supplied `license_type` is never projected (a
//     presenter never mints a License Type);
//   - non-canonical product grants are dropped;
//   - no raw email, key, token, customer row, credential, or card data field
//     exists by construction;
//   - the module is pure and holds no module-level mutable state, so it can
//     never cache local commercial policy.
//
// Deterministic and dependency-free so tests can import it directly.

/** Canonical Spec 172 License Type codes (docs/contracts/spec172-license-types.v1.yaml). */
export const SPEC172_LICENSE_TYPE_CODES = [
  'focusa_operator_lifetime_v1',
  'uiai_operator_lifetime_v1',
  'focusa_uiai_operator_bundle_lifetime_v1',
] as const;

export type Spec172LicenseTypeCode = (typeof SPEC172_LICENSE_TYPE_CODES)[number];

/** Canonical product codes that a Spec 172 grant may carry. */
export const SPEC172_PRODUCT_CODES = ['focusa', 'uiai_engine'] as const;

export type Spec172ProductCode = (typeof SPEC172_PRODUCT_CODES)[number];

/** Frozen display labels (Spec 172 §4.1 canonical names). Labels only: no
 * prices, grants, limits, or sale status are ever rendered by a presenter. */
export const LICENSE_TYPE_LABELS: Record<Spec172LicenseTypeCode, string> = {
  focusa_operator_lifetime_v1: 'Focusa Operator Lifetime v1',
  uiai_operator_lifetime_v1: 'UIAI Engine Operator Lifetime v1',
  focusa_uiai_operator_bundle_lifetime_v1: 'Focusa + UIAI Operator Lifetime Bundle',
};

export const VERIFIED_NO_LICENSE_LABEL =
  'Verified no-license limited access (no automatic expiry)';

/** Frozen node semantics (Spec 172 §7.3): one operator seat, up to three
 * registered operator nodes, and multiple approved clients on the same node
 * do not consume separate nodes. Rendering sentence only — node truth lives
 * in the authority, never in a presenter counter. */
export const SPEC172_NODE_SEMANTICS =
  'One verified operator seat and up to three registered operator nodes; CLI, TUI, Pi, menubar, Focusa Desktop, and other approved clients on the same node do not consume separate nodes.';

/** Frozen presenter-not-product sentence (Spec 172 §15). */
export const SPEC172_PRESENTER_NOT_PRODUCT =
  'Menubar and TUI are presenters, not products. They project the canonical operation decision; they never own pricing, grants, limits, or commercial policy.';

/** Retained controls that are NEVER disabled by an entitlement decision
 * (Spec 172 §5.3 / §6.2; frozen fixture shared with the TUI presenter and
 * the menubar action map `spec172.locked_state_fixtures`). */
export const SPEC172_RETAINED_CONTROLS = [
  'navigation',
  'status',
  'account',
  'read',
  'export',
  'recovery',
  'repair',
  'update',
  'uninstall',
] as const;

export type Spec172RetainedControl = (typeof SPEC172_RETAINED_CONTROLS)[number];

/** Frozen upgrade-display triggers — the ONLY presenter-accepted signals that
 * an upgrade/recovery action exists. They come from the daemon presenter
 * vocabulary (`presenter.allowed_actions`); a presenter never decides an
 * upgrade itself. */
export const SPEC172_UPGRADE_TRIGGERS = [
  'select_purchase',
  'open_checkout',
  'activate_or_manage_entitlement',
] as const;

export interface Spec172UpgradeDisplay {
  available: boolean;
  action: string;
  label: string;
  explanation: string;
}

export interface Spec172Posture {
  license_type: Spec172LicenseTypeCode | null;
  product_grants: Spec172ProductCode[];
  verified_no_license: boolean;
  upgrade: Spec172UpgradeDisplay;
  node_semantics: string;
  presenter_not_product: string;
  retained_controls: readonly Spec172RetainedControl[];
}

/** Project the daemon `GET /v1/license/status` payload onto the frozen
 * Spec 172 presenter posture. Fail closed on every unknown or
 * caller-controlled value; never mints a License Type, product grant, or
 * upgrade. */
export function projectSpec172Posture(payload: unknown): Spec172Posture {
  const record = (payload ?? {}) as Record<string, unknown>;
  const status = String(record.status ?? '').toLowerCase();
  const license_type = projectLicenseType(record.license_type);
  const product_grants = projectProductGrants(record.product_grants);
  const verified_no_license =
    String(record.posture ?? '').toLowerCase() === 'verified_no_license' ||
    status === 'unactivated' ||
    status === 'verified_no_license';
  const upgrade = projectUpgrade(record, license_type, status);
  return {
    license_type,
    product_grants,
    verified_no_license,
    upgrade,
    node_semantics: SPEC172_NODE_SEMANTICS,
    presenter_not_product: SPEC172_PRESENTER_NOT_PRODUCT,
    retained_controls: SPEC172_RETAINED_CONTROLS,
  };
}

/** Frozen locked-state accessibility fixture (Spec 172 §11.1, §5.3, §6.2):
 * the upgrade action and the always-reachable retained controls. Identical
 * rendering is enforced in the TUI presenter
 * (crates/focusa-tui/src/spec172_presenter.rs) and the menubar action map
 * (`spec172.locked_state_fixtures`). */
export function lockedStateFixture(): string {
  return `locked_state_fixtures: upgrade_action=activate_or_manage_entitlement retained_controls=[${SPEC172_RETAINED_CONTROLS.join(',')}] never_disabled=read,export,recovery,repair,update,uninstall`;
}

function projectLicenseType(value: unknown): Spec172LicenseTypeCode | null {
  const code = String(value ?? '').trim();
  return (SPEC172_LICENSE_TYPE_CODES as readonly string[]).includes(code)
    ? (code as Spec172LicenseTypeCode)
    : null; // Unknown or caller-supplied codes never project.
}

function projectProductGrants(value: unknown): Spec172ProductCode[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const grants: Spec172ProductCode[] = [];
  for (const entry of value) {
    const code = String(entry).trim();
    if (!(SPEC172_PRODUCT_CODES as readonly string[]).includes(code)) continue;
    if (seen.has(code)) continue;
    seen.add(code);
    grants.push(code as Spec172ProductCode);
  }
  return grants;
}

function projectUpgrade(
  record: Record<string, unknown>,
  license_type: Spec172LicenseTypeCode | null,
  status: string,
): Spec172UpgradeDisplay {
  const presenter = (record.presenter ?? {}) as Record<string, unknown>;
  const allowed = Array.isArray(presenter.allowed_actions)
    ? presenter.allowed_actions.map((a) => String(a))
    : [];
  const triggered = allowed.some((action) =>
    (SPEC172_UPGRADE_TRIGGERS as readonly string[]).includes(action),
  );
  // Accurate display: an actively granted License Type is managed, never
  // re-sold as an Operator upgrade by a presenter (Spec 172 §10.3).
  const activeGrant =
    license_type !== null && (status === 'active' || status === 'offline_grace');
  const available = triggered && !activeGrant;
  if (available) {
    return {
      available: true,
      action: 'activate_or_manage_entitlement',
      label: license_type ? 'Manage entitlement' : 'Operator upgrade available',
      explanation:
        'Operator is the initial License Type; the Bundle grants both the Focusa and UIAI Operator v1 types. Upgrade actions run through the authority checkout, never locally.',
    };
  }
  return {
    available: false,
    action: 'manage',
    label: 'Manage entitlement',
    explanation:
      'The current entitlement is usable. Node, lease, and account management run through the authority.',
  };
}
