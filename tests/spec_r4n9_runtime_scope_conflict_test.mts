// FOCUSA_FIX-r4n9: Runtime test for scope authority enforcement
// Tests that when scope conflict is detected, next_action is blocked

import { buildAttentionRecallVerdict, buildCurrentAskScopeVerdict } from './apps/pi-extension/src/state.ts';

console.log("=== FOCUSA_FIX-r4n9 Runtime Test ===\n");

// Test Case 1: Scope conflict scenario
console.log("Test 1: Scope conflict (authority should be blocked)");
const verdictWithConflict = buildAttentionRecallVerdict({
  workpointPacket: {
    project_root: "/home/wirebot/focusa",
    continuity_id: "focusa-cont-root-xxx",
    mission: "Test mission",
    next_slice: "Continue test work"
  },
  currentAskText: "Work on /home/other/project instead",
  projectRoot: "/home/other/project",
  continuityId: "other-cont-yyy"
});

console.log(`  action_authority_for_current_ask: ${verdictWithConflict.memory_anchor.action_authority_for_current_ask}`);
console.log(`  next_action: ${verdictWithConflict.memory_anchor.next_action}`);
console.log(`  scope_conflict_reason: ${verdictWithConflict.scope_conflict_reason}`);

const pass1 = verdictWithConflict.memory_anchor.action_authority_for_current_ask === false;
const pass2 = verdictWithConflict.memory_anchor.next_action.includes("BLOCKED");
const pass3 = verdictWithConflict.scope_conflict_reason !== "none";

if (pass1 && pass2 && pass3) {
  console.log("✓ PASS: Scope conflict blocks authority and cuts next_action\n");
} else {
  console.log(`✗ FAIL: pass1=${pass1}, pass2=${pass2}, pass3=${pass3}\n`);
  process.exit(1);
}

// Test Case 2: No scope conflict (authority should be allowed)
console.log("Test 2: No scope conflict (authority should be allowed)");
const verdictNoConflict = buildAttentionRecallVerdict({
  workpointPacket: {
    project_root: "/home/wirebot/focusa",
    continuity_id: "focusa-cont-root-xxx",
    mission: "Test mission",
    next_slice: "Continue test work"
  },
  currentAskText: "Continue with the test work",
  projectRoot: "/home/wirebot/focusa",
  continuityId: "focusa-cont-root-xxx"
});

console.log(`  action_authority_for_current_ask: ${verdictNoConflict.memory_anchor.action_authority_for_current_ask}`);
console.log(`  next_action: ${verdictNoConflict.memory_anchor.next_action}`);

const pass4 = verdictNoConflict.memory_anchor.action_authority_for_current_ask === true;
const pass5 = !verdictNoConflict.memory_anchor.next_action.includes("BLOCKED");

if (pass4 && pass5) {
  console.log("✓ PASS: No conflict allows authority and normal next_action\n");
} else {
  console.log(`✗ FAIL: pass4=${pass4}, pass5=${pass5}\n`);
  process.exit(1);
}

console.log("=== ALL RUNTIME TESTS PASSED ===");
console.log("\nFix verification:");
console.log("- Scope conflict → action_authority=false + next_action=BLOCKED");
console.log("- No conflict → action_authority=true + next_action=normal");
