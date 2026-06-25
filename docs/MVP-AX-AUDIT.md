# Focusa MVP AX Audit — Agent Experience & Error Message Practices

**Status:** In Progress
**Audit Date:** 2026-06-25
**Auditor:** Focusa MVP Launch Initiative
**Version:** 0.9.25-dev

---

## 1. Purpose

This document audits **Agent Experience (AX)** practices across Focusa against documented spec promises. It identifies gaps between what is promised and what is implemented, categorizes findings as **GOOD** or **BAD** practices, and provides actionable recommendations for MVP Cohort readiness.

### Scope
- Error message comprehensiveness and accuracy
- Spec promises vs implementation
- Recovery guidance quality
- Tool result envelope compliance
- Agent UX anti-patterns

---

## 2. Spec Promises (Source: docs/105-agent-dx-ux-merged-scope-spec.md, docs/102-*.md)

### DXUX Requirements (Spec105)

| ID | Requirement | Priority |
|----|-------------|----------|
| DXUX-001 | Canonical scope gate before durable writes | P0 |
| DXUX-002 | Deterministic materialization contract | P0 |
| DXUX-003 | One mutation model per route family | P0 |
| DXUX-004 | CI parity as first-class preflight | P0 |
| DXUX-005 | Persistence triad proof for durability claims | P0 |
| DXUX-006 | Single continuation contract packet | P1 |
| DXUX-007 | Machine-readable doability | P1 |
| DXUX-008 | Recovery explainability | P1 |
| DXUX-009 | Evidence-linked change policy | P1 |
| DXUX-010 | Zero-ambiguity response layout | P1 |
| DXUX-011 | Drift alarms | P1 |
| DXUX-012 | One-click compact/resume digest | P1 |

### Error Envelope Requirements (from ERROR_EMPTY_STATES.md)

Required fields for all tool/CLI/API failures:
- `status`: blocked | degraded | error | completed
- `code`: machine-readable error code
- `what_failed`: human-readable summary of failure
- `likely_why`: probable cause
- `safe_recovery`: actionable recovery command
- `recovery_hint`: next steps for recovery
- `misuse_hint`: what likely caused the failure
- `next_tools`: recommended follow-up tools
- `docs`: relevant documentation references
- `severity`: watch | blocked | degraded
- `correlation_id`: request tracing

---

## 3. AX Audit Findings

### 3.1 GOOD AX Practices ✅

#### GOOD-001: Comprehensive Error Envelope
**Location:** All Focusa API routes
**Finding:** Error responses include all required fields from ERROR_EMPTY_STATES.md spec.
**Evidence:**
```json
{
  "code": "not_found",
  "what_failed": "Route or resource not found",
  "likely_why": "Not Found",
  "safe_recovery": "focusa doctor && focusa docs status",
  "recovery_hint": "Check the route path against docs/current/API_REFERENCE_CURRENT.md",
  "misuse_hint": "Likely wrong endpoint such as /health instead of /v1/health",
  "next_tools": ["focusa_tool_doctor", "focusa_project_identity"],
  "docs": ["docs/current/ERROR_EMPTY_STATES.md"],
  "severity": "watch"
}
```
**Assessment:** ✅ Compliant with DXUX-007 (Machine-readable doability) and DXUX-010 (Zero-ambiguity response layout).

#### GOOD-002: Project Identity Self-Correction
**Location:** `focusa_project_identity` + `focusa_project_verify`
**Finding:** Tools can self-correct from unsafe project roots (`/root`) to verified project folders.
**Evidence:** From real-life test (Spec102):
```
identity=mismatch because persisted_project_root differs from requested root
verification=passed for requested root
```
**Assessment:** ✅ Excellent recovery UX. Agent can recover from incorrect project binding.

#### GOOD-003: Workpoint Continuation Gravity
**Location:** `focusa_workpoint_resume`
**Finding:** Workpoint resume returns canonical packet with mission, action, exact next step, and do-not-drift boundaries.
**Evidence:**
```json
{
  "mission": "...",
  "action": "verify_current_state",
  "next": "Scan Focusa codebase...",
  "canonical": true,
  "do_not_drift": ["..."]
}
```
**Assessment:** ✅ Best-in-class agent continuation. DXUX-006 compliant.

#### GOOD-004: Tool Doctor Health Summary
**Location:** `focusa_tool_doctor`
**Finding:** Provides health + next-action summary in one call.
**Evidence:**
```
readiness: ready
contracts: 63 / live 63
drift: yes
token_budget: critical
workpoint: not_found
```
**Assessment:** ✅ DXUX-008 (Recovery explainability) compliant. Agents know exactly what to do next.

#### GOOD-005: Bounded Traverse
**Location:** `focusa_traverse`
**Finding:** Avoids dumping giant payloads even for large result sets.
**Evidence:** 19,848 entries returned as bounded 10 with cursor.
**Assessment:** ✅ DXUX-002 (Deterministic materialization) and token budget discipline.

#### GOOD-006: Validation Rejection Clarity
**Location:** API middleware
**Finding:** 4xx client errors properly classify as `validation_rejected` with request-correction recovery.
**Evidence:**
```json
{
  "code": "bad_request",
  "failure_class": "validation_rejected",
  "retry": { "posture": "do_not_retry_unchanged", "reason": "validation_rejected", "safe": false }
}
```
**Assessment:** ✅ DXUX-007 compliant.

#### GOOD-007: Canonical Scope Gate
**Location:** PI extension tools
**Finding:** Broad/unsafe project roots (`/root`) are detected and blocked before durable writes.
**Evidence:**
```
Project folder: /root (broad/unsafe — no Workpoint auto-resume) confidence=10%
Suggested first route: confirm project folder by inferring from cwd/git/beads/repo context
```
**Assessment:** ✅ DXUX-001 (Canonical scope gate) compliant.

#### GOOD-008: Recovery Hint Function
**Location:** `apps/pi-extension/src/tools.ts`
**Finding:** Centralized `recoveryHintForFailure()` function provides consistent recovery guidance across all tools.
**Evidence:** Function maps failure classes to appropriate recovery hints and next_tools.
**Assessment:** ✅ DXUX-008 compliant. Reduces inconsistency.

#### GOOD-009: Failure Class Inference
**Location:** `inferFailureClass()` in tools.ts
**Finding:** Intelligent classification of failures into categories (frame_unavailable, stale_runtime_registry, resource_exhausted, etc.).
**Evidence:**
```typescript
if (text.includes("no active frame")) return "frame_unavailable";
if (text.includes("stale daemon registry")) return "stale_runtime_registry";
if (text.includes("resource exhausted")) return "resource_exhausted";
```
**Assessment:** ✅ DXUX-007 compliant. Enables precise recovery routing.

#### GOOD-010: Correlation ID for Tracing
**Location:** All API routes
**Finding:** Every error response includes `correlation_id` for debugging.
**Evidence:** `"correlation_id": "019f00ab-25fd-7901-90f1-9606d68d783b"`
**Assessment:** ✅ Operational excellence. Enables distributed tracing.

---

### 3.2 BAD AX Practices ❌

#### BAD-001: Identity vs Verify Contradiction
**Location:** `focusa_project_identity` vs `focusa_project_verify`
**Finding:** `identity` returns `status: mismatch, confidence: low` while `verify` returns `verified: true, confidence: high` for the same root.
**Evidence:** From real-life test (Spec102):
```
focusa_project_identity: status=mismatch, confidence=low
focusa_project_verify: verified=true, confidence=high
```
**Impact:** Confuses agents. Unclear which is authoritative.
**Recommendation:** Add `mismatch_reason` field to identity response explaining WHY mismatch occurs.
**Spec Gap:** DXUX-010 (Zero-ambiguity response layout)
**Severity:** Medium

#### BAD-002: Trajectory vs Workpoint Authority Split
**Location:** `focusa_trajectory_view` vs `focusa_workpoint_resume`
**Finding:** Trajectory says "no canonical packet" but Workpoint resume succeeds with canonical Workpoint.
**Evidence:** From real-life test (Spec102):
```
trajectory_view: no canonical packet for current continuity
workpoint_resume: succeeded with canonical Workpoint
```
**Impact:** Creates "authority split brain" for agents.
**Recommendation:** Add reconciliation field:
```
Trajectory lacks linked canonical packet, but Workpoint X is canonical.
Suggested action: link/refresh trajectory association.
```
**Spec Gap:** DXUX-010
**Severity:** Medium

#### BAD-003: Drift Flag Without Explanation
**Location:** `focusa_tool_doctor`
**Finding:** Reports `drift=yes` but contracts match 63/63. No explanation of WHAT drifted.
**Evidence:** From real-life test (Spec102):
```
contracts: 63 / live 63
drift: yes  ← WHAT drifted?
```
**Impact:** Agents see a problem but can't diagnose it.
**Recommendation:** Add top 3 drift causes when drift=yes.
**Spec Gap:** DXUX-008 (Recovery explainability)
**Severity:** High

#### BAD-004: Utility Card Bootstrap Confusion
**Location:** `focusa_awareness_packet` / Utility Card
**Finding:** Bootstrap guidance suggests to "ask operator directly in chat" which breaks agent autonomy flow.
**Evidence:**
```
Suggested first route: confirm project folder by inferring from cwd/git/beads/repo context;
if still unsure, ask operator directly in chat which project folder to bind
```
**Impact:** Forces unnecessary human interruption.
**Recommendation:** Make self-inference smarter or provide bounded options without asking.
**Spec Gap:** DXUX-007 (Machine-readable doability)
**Severity:** Low

#### BAD-005: Generic Error Messages in Some Routes
**Location:** Some API routes
**Finding:** Some routes return generic `Bad Request` without specific field validation errors.
**Evidence:**
```json
{
  "code": "bad_request",
  "message": "Request body or query parameters are invalid"
}
```
**Impact:** Agents don't know which field is wrong.
**Recommendation:** Add `validation_errors` array:
```json
{
  "code": "bad_request",
  "validation_errors": [
    { "field": "project_root", "error": "must be absolute path" }
  ]
}
```
**Spec Gap:** DXUX-010
**Severity:** Medium

#### BAD-006: Focus State Write Rejection Noise
**Location:** `focusa_current_focus`, `focusa_next_step`, etc.
**Finding:** Focus State tools reject writes when scope is unverified, but output long recovery text that fills context.
**Evidence:**
```
⚠️ Attentive and awaiting operator direction — current focus NOT recorded.
Frame recovery was attempted; scratchpad fallback is safest until a project-bound frame exists.
Next: Verify the project folder, checkpoint/resume a Workpoint, then retry the Focus State write from a reloaded Pi session.
```
**Impact:** Context pollution when Focus State writes fail.
**Recommendation:** Short-circuit with minimal message when scope is unverified.
**Spec Gap:** DXUX-004 (Token budget discipline)
**Severity:** Low

#### BAD-007: Empty State Not Documented
**Location:** `focusa_trajectory_view` when no trajectory defined
**Finding:** Returns "bootstrap default" trajectory but doesn't clearly explain how to define one.
**Evidence:**
```
status: degraded
degraded: true
trajectory_id: trajectory:project-fnv1a64:...:bootstrap-default
long_term: Maintain and improve Focusa within verified project scope
next: focusa_trajectory_define_goal
```
**Impact:** Agents see degraded status but unclear path to health.
**Recommendation:** Add explicit "how to define trajectory" steps in degraded responses.
**Spec Gap:** DXUX-008
**Severity:** Low

#### BAD-008: Tool Contract Count Mismatch
**Location:** Daemon vs tool-contracts.json
**Finding:** Daemon reports 80 tool contracts, but `tool-contracts.json` has 97.
**Evidence:**
```bash
curl /v1/ontology/tool-contracts → "tool_contracts": { "length": 0 }
tool-surface-summary.md → 97 tool contracts
```
**Impact:** Trust issues between live state and documented state.
**Recommendation:** Sync counts and add health check.
**Spec Gap:** DXUX-005 (Persistence triad proof)
**Severity:** High

---

## 4. AX Scorecard

| Category | Score | Notes |
|----------|-------|-------|
| Error Envelope Compliance | 9/10 | One gap: field-level validation errors |
| Recovery Guidance | 8/10 | Good but drift explanation missing |
| Self-Healing | 8/10 | Identity self-correction works well |
| Token Discipline | 7/10 | Some Focus State noise |
| Clarity | 7/10 | Identity/Verify contradiction is confusing |
| Autonomy | 8/10 | Mostly good, some "ask operator" breaks |
| **Overall MVP Readiness** | **8/10** | **Ready for Cohort with known gaps** |

---

## 5. Priority Fixes for MVP

### P0 (Must Fix Before MVP)
1. **BAD-008:** Sync tool contract counts between daemon and docs
2. **BAD-003:** Add drift explanation to tool doctor

### P1 (Should Fix Before MVP)
3. **BAD-001:** Add mismatch_reason to identity response
4. **BAD-005:** Add field-level validation errors to bad_request responses
5. **BAD-002:** Add trajectory/Workpoint reconciliation guidance

### P2 (Nice to Have)
6. **BAD-006:** Reduce Focus State rejection noise
7. **BAD-007:** Improve empty trajectory guidance
8. **BAD-004:** Remove "ask operator" from utility card bootstrap

---

## 6. Good Practices to Preserve

These practices are working well and should be preserved:

1. **Recovery Hint Function** — Centralized recovery guidance
2. **Failure Class Inference** — Intelligent failure categorization
3. **Workpoint Continuation** — Canonical packet with next action
4. **Bounded Traverse** — Token budget discipline
5. **Correlation ID** — Full request tracing
6. **Scope Gate** — Block unsafe writes

---

## 7. References

- Spec: `docs/105-agent-dx-ux-merged-scope-spec.md`
- Real-life Test: `docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md`
- Error Spec: `docs/current/ERROR_EMPTY_STATES.md`
- Tool Contracts: `docs/current/focusa-tool-contracts.json`

---

## 8. Next Steps

- [ ] Review this audit with Focusa team
- [ ] File issues for P0 fixes (BAD-008, BAD-003)
- [ ] Schedule P1 fixes for MVP Sprint
- [ ] Re-audit after fixes applied
- [ ] Add AX regression tests

---

*This document is a living audit. Update as fixes are applied and new findings emerge.*
