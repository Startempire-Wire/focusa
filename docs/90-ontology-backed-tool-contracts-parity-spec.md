# Spec90 — Ontology-Backed Focusa Tool Contracts and Parity Hardening

## 1. Purpose

Make every current `focusa_*` Pi tool maximally sharp by giving it a canonical machine-readable contract that links the tool to Focusa ontology actions, API routes, CLI parity, core/reducer surface, docs, result-envelope expectations, live health checks, and explicit exemptions.

## 2. Problem statement

The present build has 43 useful Focusa Pi tools, current docs, CLI domains, API routes, and ontology endpoints. The remaining gap is not raw feature absence; it is contract drift risk. Tool behavior, docs, API routes, CLI commands, ontology actions, and doctor checks are still partly maintained by convention.

Spec90 turns this into a self-verifying system.

## 3. Non-goals

- Do not invent future tools.
- Do not claim unfinished API/CLI parity.
- Do not expose secrets or live tokens in generated docs.
- Do not remove Pi-only tools merely because they have no CLI equivalent.
- Do not make state hygiene destructive.
- Do not require network exposure beyond the current local daemon model.

## 4. Current-build baseline

As of `v0.9.3-dev` / post-doc-audit state:

- Pi extension exposes 43 `focusa_*` tools from `apps/pi-extension/src/tools.ts`.
- Root docs include one individual doc per tool under `docs/focusa-tools/tools/`.
- API routes include current surfaces for Workpoint, Work-loop, Metacognition, Ontology, Lineage, Focus State, ECS, Events, and related domains.
- CLI exposes domains including `workpoint`, `metacognition`, `ontology`, `lineage`, `clt`, `focus`, `memory`, `ecs`, `events`, `state`, and others.
- `tool_result_v1` is already attached by the Pi extension wrapper when available.
- `focusa_tool_doctor` currently checks daemon health/workpoint/work-loop readiness, but does not yet prove full contract parity.

## 5. Core concept: Focusa Tool Contract

Each Focusa Pi tool MUST have one canonical contract entry.

### 5.1 Contract fields

Required fields:

- `name` — exact Pi tool name, e.g. `focusa_workpoint_checkpoint`.
- `family` — one of `focus_state`, `workpoint`, `work_loop`, `metacognition`, `tree_lineage`, `diagnostics_hygiene`.
- `label` — human-facing label.
- `purpose` — current-build purpose, no roadmap language.
- `ontology_action` — canonical action id, e.g. `workpoint.checkpoint`.
- `ontology_objects` — object kinds touched/read/written.
- `api_routes` — current route mappings, empty only with explicit exemption.
- `cli_commands` — current CLI mappings, empty only with explicit exemption.
- `core_surface` — current core/state/reducer area or explicit local-only surface.
- `doc_path` — individual docs path.
- `result_envelope` — required `tool_result_v1` expectations.
- `side_effect_profile` — `read_only`, `write_state`, `checkpoint`, `evidence_link`, `diagnostic`, or `local_note`.
- `parity_status` — `full`, `domain`, `pi_only`, `local_only`, or `degraded_known`.
- `exemptions` — reasons for absent API/CLI/core parity, if any.
- `live_check` — how doctor/CI checks it.

### 5.2 Parity status definitions

- `full` — Pi tool has direct ontology action, API route, CLI command, docs, and result envelope.
- `domain` — Pi tool maps to a broader API/CLI domain but not a one-command exact equivalent.
- `pi_only` — tool is intentionally Pi-local/orchestration-only.
- `local_only` — tool writes local scratchpad/session state and does not mutate daemon state.
- `degraded_known` — present but intentionally limited; contract must explain why.

## 6. Ontology integration requirements

### 6.1 Action naming

Use dotted action ids:

- `focus_state.scratch`
- `focus_state.decide`
- `workpoint.checkpoint`
- `work_loop.control`
- `metacognition.capture`
- `tree.snapshot_state`
- `diagnostics.tool_doctor`

### 6.2 Ontology projection

The current build MUST expose or document tool contracts as ontology actions. Minimum initial implementation may be a machine-readable registry consumed by docs/tests/doctor. A later implementation may project it through `/v1/ontology/actions`.

### 6.3 Object kinds

Common object kinds:

- `FocusState`
- `ScratchpadNote`
- `WorkpointResumePacket`
- `WorkLoopState`
- `MetacogSignal`
- `LineageNode`
- `TreeSnapshot`
- `EvidenceRef`
- `ToolContract`

## 7. API/CLI parity requirements

### 7.1 API parity

Every contract with `api_routes` MUST reference routes present in `crates/focusa-api/src/routes/*.rs`.

### 7.2 CLI parity

Every contract with `cli_commands` MUST reference a current CLI command/domain from `focusa --help` or command module files.

### 7.3 Exemptions

Exemptions must be explicit. Valid exemptions:

- `local_scratchpad_only`
- `pi_session_only`
- `doctor_orchestration_only`
- `domain_cli_only`
- `api_domain_only`
- `approval_placeholder`

## 8. Result envelope requirements

Every Pi tool MUST return or be wrapped with a `tool_result_v1` detail object containing, where available:

- `ok`
- `tool`
- `status`
- `canonical`
- `degraded`
- `summary`
- `retry`
- `side_effects`
- `evidence_refs`
- `next_tools`
- `error`
- `raw`

Doctor/parity validation MUST fail if a registered tool cannot be wrapped or lacks a contract.

## 9. Doctor requirements

`focusa_tool_doctor` MUST evolve from health-only diagnostics to contract-aware diagnostics.

Minimum output additions:

- `contracts_total`
- `contracts_by_family`
- `contract_coverage`
- `missing_docs`
- `missing_contracts`
- `route_coverage` when route inventory is available
- `cli_coverage` when CLI inventory is available
- `known_exemptions`

Future full-chain check:

```text
Pi tool loads -> contract exists -> doc exists -> ontology action exists -> API route exists or exemption -> CLI command exists or exemption -> result envelope valid -> live call healthy or blocked with known reason
```

## 10. Validation script requirements

Add a deterministic validation script that can run without daemon availability.

Required checks:

1. Parse current Pi tools from `apps/pi-extension/src/tools.ts`.
2. Load canonical contract registry.
3. Assert every tool has exactly one contract.
4. Assert no contract references a non-existent tool.
5. Assert every contract doc path exists.
6. Assert every contract has ontology action and object kinds.
7. Assert missing API/CLI parity has explicit exemptions.
8. Assert docs README/root README link coverage remains intact.
9. Emit JSON and human-readable output.

## 11. Documentation requirements

Update docs with only current-build truth:

- Spec90 file.
- Current tool contract registry doc.
- Root README link to Spec90 and registry doc.
- `docs/README.md` link to Spec90 and registry doc.
- Changelog entry for Spec90 initial implementation.
- Evidence file for validation.

## 12. Bead decomposition requirements

Spec90 MUST be decomposed into beads before implementation work proceeds beyond the initial foundation.

Minimum beads:

1. Spec90 authoring and acceptance checklist.
2. Tool contract registry foundation.
3. Contract validation script.
4. Doctor contract-awareness upgrade.
5. Docs/README/changelog updates.
6. API/CLI/ontology projection follow-up.
7. Uniform result envelope hardening follow-up.
8. Live daemon full-chain proof follow-up.

## 13. Acceptance criteria

Initial implementation is complete when:

- Spec90 exists and is linked.
- A machine-readable contract registry exists for all 43 current tools.
- Validation script passes with 43/43 tool contract coverage.
- Every contract doc path exists.
- Every missing API/CLI mapping has an explicit exemption.
- `focusa_tool_doctor` reports contract coverage from the registry.
- README and docs index link the Spec90/registry docs.
- Evidence file records validation commands/results.
- Beads reflect the decomposed work and completed initial implementation.

Full implementation is complete when:

- Ontology action projection consumes the same registry.
- API route mappings are validated against generated route inventory.
- CLI mappings are validated against generated CLI inventory.
- Tool docs include generated contract tables/parameter references.
- Doctor can prove the full chain live when daemon is available.
- CI enforces contract/doc/tool parity.

## 14. Safety and privacy

- No keys, tokens, or credentials in generated docs/evidence.
- Use placeholder env var names only.
- Continue using `guardian scan` for docs/skills before release.
- Do not persist raw logs when evidence refs are sufficient.

## 15. Implementation sequence

1. Create this spec and beads.
2. Add `apps/pi-extension/src/tool-contracts.ts`.
3. Add `scripts/validate-focusa-tool-contracts.mjs`.
4. Update `focusa_tool_doctor` to include registry/coverage diagnostics.
5. Generate/write `docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md`.
6. Update README, docs index, changelog.
7. Run validation, TypeScript compile, targeted docs secret scan.
8. Commit and push initial Spec90 foundation.
