# Cross-Project Scope Hard-Stop (GitHub #2)

**Status:** Implemented  
**Spec:** §105 DXUX-001, DXUX-007, DXUX-011  
**Refs:** focusa-r4n9, focusa-xp8i

---

## Problem

Stale Focusa context from a different project caused agents to switch projects despite `scope_conflict` warnings. The Workpoint/Trajectory resume output continued to provide executable `next_action` even when scope mismatched.

## Desired Behavior

When current ask project scope conflicts with resumed Workpoint/Trajectory project scope:
- `action_authority_for_current_ask=false` blocks execution
- Workpoint resume returns `rejected_scope_mismatch` status
- Focus Slice renders `next_action=BLOCKED` (no escape hatch)
- Agent cannot proceed without `focusa_project_verify` + `focusa_workpoint_checkpoint`

## Implementation

### 1. Reducer Layer (Rust)

`apps/pi-extension/src/state.ts:1063` — `buildAttentionRecallVerdict()`:

```typescript
next_action: conflictReason
  ? "BLOCKED: scope conflict — verify project scope with focusa_project_identity before continuing"
  : boundedAttentionText(nextAction, 180),
```

### 2. Focus Slice Rendering

`apps/pi-extension/src/state.ts:1077` — `formatAttentionRecallFocusSliceLines()`:

```
⛔ next_action=BLOCKED: scope conflict — verify project scope...  ← EXECUTION BLOCKED
```

### 3. Workpoint Resume Rejection

`crates/focusa-api/src/routes/workpoint.rs` — `/v1/workpoint/resume`:

```json
{
  "status": "rejected_scope_mismatch",
  "canonical": false,
  "canonical_for_requested_scope": false,
  "scope_found": false,
  "safe_recovery": "ignore this resume packet; follow latest operator instruction",
  "next_step_hint": "create a new Workpoint checkpoint in the current project before trusting resume"
}
```

### 4. Project Identity Auto-Recovery

`apps/pi-extension/src/state.ts:855` — `searchProjectMarkerForAlias()`:

When a project alias is mentioned but no project_root is stored, search filesystem for `.focusa-project.json` to resolve canonical root.

## Acceptance Criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `scope_conflict` + `action_authority=false` blocks continuation | ✅ | tests/e2e_scope_authority_enforcement_test.sh |
| Resume returns `blocked/no-executable-next-action` | ✅ | tests/e2e_stale_carryover_hard_stop_test.sh |
| Operator confirmation required before cross-project continuation | ✅ | Focus Slice ⛔ indicator |
| Regression test for stale Focusa carryover | ✅ | tests/e2e_stale_carryover_hard_stop_test.sh |
| Docs/tool guidance updated | ✅ | This document |

## Agent Recovery Path

When agent receives `⛔ EXECUTION BLOCKED`:

1. **Verify project identity:**
   ```bash
   focusa_project_identity project_root=/current/project
   ```

2. **Re-checkpoint in correct scope:**
   ```bash
   focusa_workpoint_checkpoint \
     project_root=/current/project \
     continuity_id=<new-continuity> \
     mission="<current task>"
   ```

3. **Resume from new canonical packet:**
   ```bash
   focusa_workpoint_resume \
     project_root=/current/project \
     continuity_id=<new-continuity>
   ```

## Anti-Patterns (Forbidden)

- ❌ Ignoring `next_action=BLOCKED` and proceeding with fallback chain
- ❌ Resuming a Workpoint across project boundaries without verification
- ❌ Using transcript tail as authority for cross-project continuation
- ❌ Treating `canonical_for_saved_scope` as canonical for the current ask

## Related Specs

- §105 DXUX-001 (Canonical scope gate)
- §105 DXUX-007 (Machine-readable doability)
- §105 DXUX-011 (Drift alarms)
- §92 polish-hooks-efficiency-spec

## Tests

```bash
# E2E: Verify scope mismatch blocks resume
bash tests/e2e_scope_authority_enforcement_test.sh

# E2E: Verify stale carryover hard-stops
bash tests/e2e_stale_carryover_hard_stop_test.sh

# Static: Verify state.ts changes
bash tests/spec_r4n9_scope_authority_enforcement_test.sh
```
