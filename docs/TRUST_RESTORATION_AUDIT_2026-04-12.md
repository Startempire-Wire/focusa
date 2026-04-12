# Trust Restoration Audit — 2026-04-12

## Scope

Forensic verification of:
- amended docs
- bead closure claims
- current implementation
- current observable runtime/CLI behavior

Goal: restore trust by replacing claim-based completion with evidence-based status.

---

## Method

Evidence sources used:
- `apps/pi-extension/src/turns.ts`
- `docs/51-ontology-expression-and-proxy.md`
- `docs/54a-operator-priority-and-subject-preservation.md`
- `docs/54b-context-injection-and-attention-routing.md`
- `docs/INDEX.md`
- `.beads/issues.jsonl`
- `tests/channel_separation_test.sh`
- `tests/pi_extension_contract_test.sh`
- live CLI/runtime observations from current daemon/session state

Trust classes:
- **VERIFIED** — docs, code, and runtime agree
- **DOCS-ONLY** — docs updated, runtime not aligned
- **TEST-ONLY** — tests/claims exist, but they do not prove actual requirement
- **FALSE-CLOSE / STALE-CLOSE** — bead says done but current repo evidence contradicts or materially undercuts completion
- **UNKNOWN** — not enough evidence collected in this pass

---

## Executive Summary

Trust in bead closure and implementation-complete claims is currently **partially compromised**.

Main failure mode:
- newer docs establish operator priority + non-coercive adoption
- current Pi extension hot path still injects behavioral coaxing and always-on full focus state
- closed beads/tests do not prove compliance with the newer requirement

Conclusion:
- **do not treat closed beads as completion evidence without runtime verification**
- for amended ontology/Pi-alignment docs, current status is **DOCS-ONLY**, not implementation-complete

---

## Critical Findings

### F1. Operator-priority docs are not implemented in the Pi hot path

#### Docs
- `docs/54a-operator-priority-and-subject-preservation.md`
  - operator newest input must remain primary
  - Focusa may guide, not hijack
- `docs/54b-context-injection-and-attention-routing.md`
  - no large always-on block competing with newest operator input
  - inject minimal applicable slice after operator-input interpretation
- `docs/51-ontology-expression-and-proxy.md`
  - operator-intent classification precedes slice assembly
  - no always-on full focus-state injection as default payload

#### Current implementation
- `apps/pi-extension/src/turns.ts`
  - `before_agent_start` appends `## Focusa Cognitive Governance (Active)` behavioral instructions
  - `context` handler comment says: `inject live Focus State before EVERY LLM call`
  - builds `[Focusa Focus State — 10-slot live refresh]`
  - prepends that block as a **user** message on every call whenever Focusa is available and `activeFrameId` exists

#### Trust classification
- **DOCS-ONLY** for the new operator-priority / non-coercive adoption requirements
- prior completion claims in this area are **FALSE-CLOSE / STALE-CLOSE**

---

### F2. `docs/INDEX.md` is stale relative to amended docs

#### Evidence
Current `docs/INDEX.md` still says:
- `55 canonical docs`

Visible index content does not canonize/list the new ontology/Pi-alignment docs in the main set, including:
- `51-ontology-expression-and-proxy.md`
- `54a-operator-priority-and-subject-preservation.md`
- `54b-context-injection-and-attention-routing.md`

#### Trust classification
- **DOCS STALE**
- any claim that the amended doc set is fully integrated/canonized is **not yet trustworthy**

---

### F3. Existing tests do not prove the new behavior

#### `tests/channel_separation_test.sh`
This test checks:
- health visibility
- SSE/internal channel presence
- anti-echo leakage
- bounded prompt assembly
- observability channels

It does **not** assert:
- operator-intent classification before slice assembly
- suppression of irrelevant injected focus state after steering change
- absence of always-on full focus-state injection in Pi extension
- non-coercive adoption / no prompt-level coaxing

#### `tests/pi_extension_contract_test.sh`
This test checks availability of API surfaces and data channels.
It does **not** prove:
- operator priority over injected Focusa state
- minimal applicable slice behavior
- removal/suppression of global behavioral coaxing

#### Trust classification
- **TEST-ONLY** for some completion claims
- these tests are insufficient evidence for the amended ontology/operator-priority requirements

---

## Bead Trust Assessment

### High-confidence false/stale close candidates

#### `focusa-bvet`
- Title: `Pi CRITICAL: add Focusa behavioral instructions to system prompt`
- Current reality: behavior exists in code
- New reality: amended docs make this pattern part of the problem, not the completion target
- Classification: **STALE-CLOSE**
- Action: supersede/reopen under new acceptance criteria

#### `focusa-zedk`
- Title: `Pi CRITICAL: define injection layering rules (prevent double-injection)`
- Current reality: code still unconditionally prepends full 10-slot focus block every LLM call
- Classification: **FALSE-CLOSE / INCOMPLETE**
- Action: reopen

#### `focusa-mj5r`
- Title: `CI-TEST-5: SPEC-54/54a visible output boundary — channel split, anti-echo, steering detection`
- Current reality: test does not prove operator-priority / subject-preservation semantics in Pi extension
- Classification: **TEST-ONLY / STALE-CLOSE**
- Action: reopen or create successor bead for real operator-priority runtime tests

#### `focusa-fx81`
- Title: `CI-TEST-2: SPEC-53 behavioral alignment tests — constraint/decision consultation, distillation, prohibitions`
- Current reality: may verify older behavioral alignment assumptions; does not prove non-coercive adoption or minimal-slice routing
- Classification: **TEST-ONLY / STALE-CLOSE**
- Action: review + likely successor bead

#### `focusa-y5jg`
- Title: `CI-TEST: Bring Focusa CI and testing up to spec per SPEC-52 through SPEC-57`
- Current reality: CI improved, but not enough to guarantee amended ontology/operator-priority compliance
- Classification: **PARTIALLY VERIFIED / OVERBROAD CLOSE**
- Action: split into verified-complete vs reopened residual gaps

#### `focusa-syh`
- Title: `Focusa cognitive loop: spec-faithful wiring [AUDIT 2026-03-30]`
- Current reality: “spec-faithful” is too broad to trust without restating exact verified surfaces
- Classification: **OVERBROAD CLOSE / NEEDS DECOMPOSITION**
- Action: do not trust as blanket completion evidence

---

## Beads appearing trustworthy in this pass

These have direct supporting runtime evidence from the current repo/CLI and are not contradicted by the amended docs reviewed here:

- `focusa-0cmg` — tool usage tracking
- `focusa-29tp` — user input signals to Focus Gate
- `focusa-0u8t` — turn/append streaming
- `focusa-0ju` — constitution loading
- `focusa-0oa` — SSE streaming functional

Classification:
- **VERIFIED (narrow scope only)**

Important: verified here means the narrow claim appears true, not that adjacent architecture is fully trustworthy.

---

## Trust Matrix

| Area | Docs | Beads | Code | Runtime | Trust |
|---|---|---|---|---|---|
| Operator priority | amended | closed claims nearby | contradicted | contradicted by behavior | FALSE-CLOSE / DOCS-ONLY |
| Non-coercive adoption | amended | closed claims nearby | contradicted | contradicted by behavior | FALSE-CLOSE / DOCS-ONLY |
| Minimal slice injection | amended | implied complete | contradicted | not observable as implemented | FALSE-CLOSE / DOCS-ONLY |
| Behavioral coaxing removal | required by amended direction | no trustworthy close | still present | still present | NOT IMPLEMENTED |
| Tool usage telemetry | supported | closed | present | observed | VERIFIED |
| Input→gate signals | supported | closed | present | observed | VERIFIED |
| Streaming append | supported | closed | present | observed | VERIFIED |
| Doc canon/index integration | incomplete | implied by doc maturity | stale | n/a | STALE |

---

## Trust Restoration Rules Going Forward

### New completion standard
No bead may be considered complete unless it is tagged with one of these evidence levels:
- `docs_only`
- `code_present`
- `test_verified`
- `runtime_verified`
- `spec_faithful`

Only `runtime_verified` or `spec_faithful` should count as actually complete.

### Required proof for spec-faithful completion
For hot-path runtime behavior beads:
1. code pointer(s)
2. test proving the exact requirement
3. runtime demonstration from CLI/integration path
4. contradiction check against latest amended docs

### Closure hygiene
If docs are amended after a bead closes:
- bead must be re-reviewed
- if acceptance criteria changed materially, bead becomes **stale-close** automatically until reconfirmed

---

## Reopen / Successor Bead Recommendations

### Reopen now
1. `focusa-zedk`
   - New reason: current Pi extension still performs unconditional full-state injection in hot path; layering not compliant with operator-priority docs

2. `focusa-mj5r`
   - New reason: current test suite does not prove 54a/54b runtime semantics; anti-echo/channel separation is not enough

3. `focusa-fx81`
   - New reason: behavioral alignment tests do not prove non-coercive adoption or operator-first routing

### Supersede / replace
4. `focusa-bvet`
   - Replace with: `Pi: remove coercive behavioral prompt injection; ambient/non-coercive adoption only`

5. Create successor bead: `Pi: operator-intent classification precedes Focusa slice assembly`

6. Create successor bead: `Pi: replace always-on 10-slot focus refresh with minimal applicable slice`

7. Create successor bead: `Docs: canonize ontology/Pi-alignment docs in docs/INDEX.md`

8. Create successor bead: `CI: add failing tests for operator-priority, steering reset, and anti-hijack slice suppression`

---

## Immediate Operational Recommendation

Until this reopen set is addressed, do **not** accept claims like:
- “implementation completed”
- “spec-faithful”
- “all gaps closed”

for the amended ontology/operator-priority area.

Use this wording instead:
- docs updated
- partial legacy implementation remains
- runtime not yet aligned with amended operator-priority requirements

---

## Bottom Line

The repo contains real progress and some verified implementation.
But for the amended docs around operator priority, subject preservation, and context injection:

- **docs are ahead of implementation**
- **some closed beads are stale or falsely reassuring**
- **trust must be restored by reopen + runtime-proof, not by prior close state**
