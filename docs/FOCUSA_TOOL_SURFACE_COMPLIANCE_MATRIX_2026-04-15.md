# Focusa Tool Surface Compliance Matrix — 2026-04-15

Scope: Pi extension tool surface vs authoritative specs (`docs/44-pi-focusa-integration-spec.md`, `docs/55-tool-action-contracts.md`, `docs/G1-07-ascc.md`).

Status legend:
- PASS = implemented with source-backed behavior
- PARTIAL = implemented but incomplete against spec authority or evidence requirements
- GAP = missing required behavior/tests/docs

## Authority anchors

- ASCC slot model: `docs/G1-07-ascc.md:28-40`
- Tool idempotency classes: `docs/55-tool-action-contracts.md:108-111`
- Metacognition tools + fallback + outage gating: `docs/44-pi-focusa-integration-spec.md:302-304`, `:389`, `:2335-2364`
- Continuous loop bridge tools: `docs/44-pi-focusa-integration-spec.md:2064-2075`, `docs/79-focusa-governed-continuous-work-loop.md:983`

## Tool-by-tool matrix

| Tool | Spec target | Implementation evidence | Test evidence | Status | Gap / action |
|---|---|---|---|---|---|
| `focusa_scratch` | Working notes in scratchpad, not Focus State (`55:111`) | Registered in `apps/pi-extension/src/tools.ts:248`; writes `/tmp/pi-scratch` via `appendScratchpadLine` (`tools.ts:27-41`) | No direct tool contract test found | PARTIAL | Add explicit success + append semantics tests (`focusa-irf4.5`) |
| `focusa_decide` | decisions slot + validation + fallback (`44:302`, `44:389`, `55:110`) | `tools.ts:286`; validation `validateDecision` (`:90-109`); fallback `mirrorFailedFocusWrite` call (`:315-319`) | Integration checks pass (`tests/pi_extension_contract_test.sh`), but no direct tool action test table | PARTIAL | Add strict per-tool success/reject/offline/recovery tests (`focusa-irf4.5`) |
| `focusa_constraint` | constraints slot + validation + fallback (`44:303`, `44:389`, `55:110`) | `tools.ts:344`; validation `validateConstraint` (`:111-124`); fallback (`:371-375`) | Same as above | PARTIAL | Same test gap (`focusa-irf4.5`) |
| `focusa_failure` | failures slot + validation + fallback (`44:304`, `44:389`, `55:110`) | `tools.ts:389`; validation `validateFailure` (`:126-141`); fallback (`:416-420`) | Same as above | PARTIAL | Same test gap (`focusa-irf4.5`) |
| `focusa_intent` | `intent` slot (`G1-07:33`) + conditional idempotency (`55:108`) | `tools.ts:433`; length guard + `pushDelta({intent})` (`:439-445`) | No dedicated tool test | PARTIAL | Validation parity + dedicated tests (`focusa-irf4.2`, `.5`) |
| `focusa_current_focus` | `current_focus` slot (`G1-07:34`) + conditional idempotency (`55:108`) | `tools.ts:453`; length guard + `pushDelta({current_focus})` (`:459-465`) | No dedicated tool test | PARTIAL | Validation parity + dedicated tests (`focusa-irf4.2`, `.5`) |
| `focusa_next_step` | `next_steps` slot (`G1-07:37`) + append semantics (`55:109`) | `tools.ts:472`; length guard + `pushDelta({next_steps:[step]})` (`:478-484`) | No dedicated tool test | PARTIAL | Validation parity + append/idempotency tests (`focusa-irf4.2`, `.5`) |
| `focusa_open_question` | `open_questions` slot (`G1-07:36`) + append semantics (`55:109`) | `tools.ts:490`; length guard + `pushDelta({open_questions:[question]})` (`:496-502`) | No dedicated tool test | PARTIAL | Validation parity + append/idempotency tests (`focusa-irf4.2`, `.5`) |
| `focusa_recent_result` | `recent_results` slot (`G1-07:38`) + append semantics (`55:109`) | `tools.ts:509`; length guard + `pushDelta({recent_results:[result]})` (`:515-521`) | No dedicated tool test | PARTIAL | Validation parity + append/idempotency tests (`focusa-irf4.2`, `.5`) |
| `focusa_note` | `notes` slot (`G1-07:40`) + append semantics (`55:109`) | `tools.ts:528`; length guard + `pushDelta({notes:[note]})` (`:534-540`) | No dedicated tool test | PARTIAL | Validation parity + bounded-decay behavior tests (`focusa-irf4.2`, `.5`) |

## Cross-cutting compliance checks

| Requirement | Evidence | Status | Bead |
|---|---|---|---|
| Missing-frame recovery for writes | `pushDelta` calls `ensurePiFrame()` when frame missing (`tools.ts:197-205`) | PASS | monitor under `focusa-irf4.3` |
| Scratch fallback for unrecoverable writes | implemented only for decision/constraint/failure (`tools.ts:315-320`, `371-375`, `416-420`) matching `44:389` | PASS (for specified tools) | verify scope under `focusa-irf4.3` |
| Outage gating of metacog write tools | tool activation filtered in `apps/pi-extension/src/session.ts:246-259` | PASS | expand deterministic tests in `focusa-irf4.4/.5` |
| Idempotency documentation parity | explicit table in `docs/55-tool-action-contracts.md:108-111` | PASS (docs) | enforce in tests `focusa-irf4.5` |
| Continuous-loop bridge tool spec alignment | tools registered `tools.ts:547-703`; spec sections `44:2064+`, `79:983` | PASS | add route/behavior tests as needed in `focusa-irf4.5` |

## Overall position

- Tool surface is implemented and usable.
- Full spec-authority compliance is not yet proven because per-tool strict contract tests are missing and validation parity for non-metacognition slots remains lighter than decision/constraint/failure quality gates.

## Execution order (mapped to parent epic `focusa-irf4`)

1. `focusa-irf4.2` — validation parity hardening
2. `focusa-irf4.3` — fallback/recovery consistency audit
3. `focusa-irf4.4` — outage gating determinism
4. `focusa-irf4.5` — strict tool contract tests
5. `focusa-irf4.6` — docs final alignment
6. `focusa-irf4.7` — final pass/fail compliance ledger
