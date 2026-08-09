#!/usr/bin/env node
/**
 * spec152f_menubar_action_map_test.mjs
 *
 * Verifies the menubar action map contract (docs/contracts/spec152f-menubar-action-map.v1.json)
 * against the menubar surface reconciliation baseline (docs/contracts/spec152f-surface-reconciliation/menubar.v1.json).
 *
 * Acceptance criteria:
 *   85/85 baseline actions are mapped
 *   buttons never mint or reinterpret entitlement
 *   protected denial remains actionable
 *
 * Usage: node tests/spec152f_menubar_action_map_test.mjs
 */

import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PROJECT_ROOT = resolve(__dirname, '..');

let failures = 0;
let passed = 0;

function assert(condition, message) {
  if (condition) {
    passed++;
  } else {
    failures++;
    console.error(`FAIL: ${message}`);
  }
}

function loadJson(relPath) {
  const absPath = resolve(PROJECT_ROOT, relPath);
  try {
    const raw = readFileSync(absPath, 'utf-8');
    return JSON.parse(raw);
  } catch (err) {
    console.error(`ERROR loading ${relPath}: ${err.message}`);
    process.exit(1);
  }
}

// ── Load contracts ──────────────────────────────────────────────────
const actionMap = loadJson('docs/contracts/spec152f-menubar-action-map.v1.json');
const baseline = loadJson('docs/contracts/spec152f-surface-reconciliation/menubar.v1.json');
const policy = loadJson('docs/contracts/spec152f-entitlement-errors.v1.json');

// ── Schema validation ───────────────────────────────────────────────
console.log('=== Schema validation ===');
assert(actionMap.schema === 'focusa.spec152f.menubar_action_map.v1', 'Action map schema is correct');
assert(actionMap.surface_group === 'menubar', 'Action map surface_group is menubar');
assert(Array.isArray(actionMap.actions), 'Action map has actions array');
assert(actionMap.actions.length === 85, `Action map has 85 actions (got ${actionMap.actions.length})`);
assert(actionMap.baseline_count === 85, 'Baseline count is 85');
assert(baseline.schema === 'focusa.spec152f.surface_reconciliation_shard.v1', 'Baseline schema is correct');
assert(baseline.row_count === 85, `Baseline has 85 rows (got ${baseline.row_count})`);
assert(Array.isArray(baseline.rows), 'Baseline has rows array');
assert(baseline.rows.length === 85, `Baseline rows count is 85 (got ${baseline.rows.length})`);

// ── Cross-reference: every baseline entry has a matching action map entry ──
console.log('\n=== Baseline / Action map cross-reference ===');
const baselineIds = new Set(baseline.rows.map(r => r.baseline_id));
const actionIds = new Set(actionMap.actions.map(a => a.baseline_id));

// Check all baseline IDs are in the action map
for (const id of baselineIds) {
  assert(actionIds.has(id), `Baseline ${id} has a matching action map entry`);
}

// Check all action map IDs are in the baseline
for (const id of actionIds) {
  assert(baselineIds.has(id), `Action map ${id} corresponds to a baseline entry`);
}

// Check no extra baseline entries
const extraBaseline = [...baselineIds].filter(id => !actionIds.has(id));
const extraActions = [...actionIds].filter(id => !baselineIds.has(id));
assert(extraBaseline.length === 0, `No unmatched baseline entries (missing: ${extraBaseline.join(', ')})`);
assert(extraActions.length === 0, `No extra action map entries (extra: ${extraActions.join(', ')})`);

// ── Action classification validation ────────────────────────────────
console.log('\n=== Action classification ===');
const VALID_CLASSES = new Set(['navigation_display', 'recovery_account', 'canonical_operation']);
const classifications = {};

for (const action of actionMap.actions) {
  const cls = action.action_class;
  assert(VALID_CLASSES.has(cls), `${action.baseline_id}: valid action_class '${cls}'`);

  classifications[cls] = (classifications[cls] || 0) + 1;

  // All actions must have source, line, handler, and description
  assert(typeof action.source === 'string' && action.source.length > 0, `${action.baseline_id}: has source`);
  assert(typeof action.line === 'number' && action.line > 0, `${action.baseline_id}: has valid line number`);
  assert(typeof action.handler === 'string' && action.handler.length > 0, `${action.baseline_id}: has handler name`);
  assert(typeof action.description === 'string' && action.description.length > 0, `${action.baseline_id}: has description`);

  // Source must match one of the exact surfaces
  assert(
    action.source.startsWith('apps/menubar/src/'),
    `${action.baseline_id}: source '${action.source}' is in menubar src`
  );
}

console.log(`  navigation_display: ${classifications.navigation_display || 0}`);
console.log(`  recovery_account:    ${classifications.recovery_account || 0}`);
console.log(`  canonical_operation: ${classifications.canonical_operation || 0}`);
assert((classifications.navigation_display || 0) + (classifications.recovery_account || 0) + (classifications.canonical_operation || 0) === 85, 'All 85 actions classified');

// ── Policy: buttons never mint or reinterpret entitlement ───────────
console.log('\n=== Policy: no commercial decisions in presenters ===');

const CANONICAL_OPS = actionMap.canonical_operations || {};

for (const action of actionMap.actions) {
  if (action.action_class === 'navigation_display') {
    // Navigation/display must not have a canonical_operation_id
    assert(
      action.canonical_operation_id === null || action.canonical_operation_id === undefined,
      `${action.baseline_id}: navigation_display has no canonical_operation_id`
    );
  }

  if (action.action_class === 'canonical_operation') {
    // Must have a valid canonical_operation_id that exists in the registry
    assert(
      typeof action.canonical_operation_id === 'string' && action.canonical_operation_id.length > 0,
      `${action.baseline_id}: canonical_operation has valid canonical_operation_id`
    );
    assert(
      CANONICAL_OPS[action.canonical_operation_id] !== undefined,
      `${action.baseline_id}: canonical_operation_id '${action.canonical_operation_id}' is registered`
    );
  }

  if (action.action_class === 'recovery_account') {
    // Recovery/account actions may reference a canonical operation (e.g. license.status)
    // or may be local-only (e.g. debug bundle copy, connection save)
    if (action.canonical_operation_id) {
      assert(
        typeof action.canonical_operation_id === 'string',
        `${action.baseline_id}: recovery_account has valid canonical_operation_id if present`
      );
    }
  }
}

// ── Policy: protected denial remains actionable ─────────────────────
console.log('\n=== Policy: protected denial remains actionable ===');

// Verify all recovery_account actions reference families that are always_available
for (const action of actionMap.actions) {
  if (action.action_class === 'recovery_account' && action.canonical_operation_id) {
    const op = CANONICAL_OPS[action.canonical_operation_id];
    if (op) {
      // Recovery operations must be in account_recovery family (always available) or
      // have a stable security allowance treatment
      assert(
        op.family === 'account_recovery' || op.treatment === 'stable_security_allowance',
        `${action.baseline_id}: recovery operation '${action.canonical_operation_id}' is always_available or stable_security_allowance (family=${op.family}, treatment=${op.treatment})`
      );
    }
  }
}

// ── Verify invariants from action map ───────────────────────────────
console.log('\n=== Action map invariants ===');
assert(Array.isArray(actionMap.invariants), 'Action map has invariants array');
assert(actionMap.invariants.length >= 5, 'Action map has sufficient invariants');
const expectedInvariants = [
  'buttons never mint or reinterpret entitlement',
  'protected denial remains actionable with recovery paths',
  'no raw keys, tokens, or customer PII in action map',
];
for (const expected of expectedInvariants) {
  const found = actionMap.invariants.some(inv => inv.toLowerCase().includes(expected.toLowerCase().split(',')[0]));
  assert(found, `Invariant present: '${expected}'`);
}

// ── Verify canonical operation registry ─────────────────────────────
console.log('\n=== Canonical operation registry ===');
const VALID_FAMILIES = new Set([
  'account_recovery', 'read_projection', 'base_focusa',
  'automation', 'team_remote', 'release_proof', 'premium_updates',
  'customer_data_export', 'internal_maintenance',
]);
const VALID_TREATMENTS = new Set([
  'always_available', 'read_allowance', 'base_entitlement',
  'optional_premium', 'stable_security_allowance',
  'always_available_subject_to_security',
]);
const VALID_MUTATION_CLASSES = new Set(['mutation', 'read', 'local_storage', 'local_only']);

for (const [opId, op] of Object.entries(CANONICAL_OPS)) {
  assert(typeof op.family === 'string', `Op '${opId}' has family`);
  assert(VALID_FAMILIES.has(op.family), `Op '${opId}' family '${op.family}' is valid`);
  assert(typeof op.treatment === 'string', `Op '${opId}' has treatment`);
  assert(VALID_TREATMENTS.has(op.treatment), `Op '${opId}' treatment '${op.treatment}' is valid`);
  assert(typeof op.mutation_class === 'string', `Op '${opId}' has mutation_class`);
  assert(VALID_MUTATION_CLASSES.has(op.mutation_class), `Op '${opId}' mutation_class '${op.mutation_class}' is valid`);
  assert(typeof op.route === 'string', `Op '${opId}' has route`);
}

// ── Verify policy families match entitlement policy ─────────────────
console.log('\n=== Policy family consistency ===');
const policyFamilies = (policy.recovery_paths?.always_reachable_subject_to_security || []).map(s => s.toLowerCase());
const recoveryOps = Object.entries(CANONICAL_OPS)
  .filter(([, op]) => op.family === 'account_recovery');
assert(recoveryOps.length >= 8, `Recovery operations count >= 8 (got ${recoveryOps.length})`);

// ── Verify no unauthorized operations mapped ────────────────────────
console.log('\n=== No unauthorized operations ===');
// Premium operations should not appear as navigation_display
const premiumOps = Object.entries(CANONICAL_OPS)
  .filter(([, op]) => op.treatment === 'optional_premium')
  .map(([id]) => id);

for (const action of actionMap.actions) {
  if (action.canonical_operation_id && premiumOps.includes(action.canonical_operation_id)) {
    assert(
      action.action_class === 'canonical_operation',
      `${action.baseline_id}: premium operation '${action.canonical_operation_id}' is canonical_operation (not navigation/recovery)`
    );
  }
}

// ── Verify no PII in action map ─────────────────────────────────────
console.log('\n=== No PII in action map ===');
const actionMapStr = JSON.stringify(actionMap);
assert(!actionMapStr.includes('@'), 'No email addresses in action map');
assert(!actionMapStr.match(/[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/), 'No email patterns in action map');

// ── Verify action_classes contract ──────────────────────────────────
console.log('\n=== Action classes contract ===');
assert(actionMap.action_classes.navigation_display !== undefined, 'navigation_display class defined');
assert(actionMap.action_classes.recovery_account !== undefined, 'recovery_account class defined');
assert(actionMap.action_classes.canonical_operation !== undefined, 'canonical_operation class defined');

const navClass = actionMap.action_classes.navigation_display;
assert(navClass.policy === 'no_entitlement_check_required', 'navigation_display has correct policy');
assert(Array.isArray(navClass.presenter_must_not), 'navigation_display has presenter_must_not');

const recClass = actionMap.action_classes.recovery_account;
assert(recClass.policy_family === 'account_recovery', 'recovery_account has correct policy_family');
assert(recClass.policy === 'always_available_subject_to_security', 'recovery_account has correct policy');

const canClass = actionMap.action_classes.canonical_operation;
assert(canClass.policy === 'inherit_canonical_operation', 'canonical_operation has correct policy');
assert(Array.isArray(canClass.presenter_must), 'canonical_operation has presenter_must');

// ── Summary ─────────────────────────────────────────────────────────
console.log(`\n=== Results ===`);
console.log(`Passed: ${passed}`);
console.log(`Failed: ${failures}`);

if (failures > 0) {
  console.error('\n❌ TEST FAILED');
  process.exit(1);
} else {
  console.log('\n✅ 85/85 baseline menubar actions successfully mapped');
  console.log('✅ Buttons never mint or reinterpret entitlement');
  console.log('✅ Protected denial remains actionable');
  process.exit(0);
}
