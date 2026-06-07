# Focusa Bloatgaurd

Status: iterable-spec-v0
Scope: design first; no implementation authority
Canonical label: Focusa Bloatgaurd (operator-provided name for the full bloat budget, dead-code, audit, and adaptive routing spec)

## 1. Core invariant

Focusa Bloatgaurd treats leanness as a first-class product invariant: rich context lives in indexed handles, evidence stores, and compact packets rather than prompts, docs, shell sprawl, duplicated proof lists, or raw tool-output dumps.

## 2. Source inspirations

Internal foundation:
- Spec100 Context Cognition (`docs/100-context-cognition-spec.md`) defines the advisory ContextCognitionPacket, Context Curator, token-budgeted selected context, excluded-context reasons, evidence/rehydrate refs, and cross-surface render contracts. Bloatgaurd must reuse that architecture rather than create a competing context engine.

External inspirations:
- `rtk`: filter and compress command/tool output before the model sees it.
- `context-mode`: sandbox raw output, store/index externally, retrieve relevant snippets only.
- Cloudflare Code Mode: ask the model to write code against APIs and return final computed results instead of every intermediate tool call.
- NerfGuard pattern: a local gateway can classify agent requests and route them to a cheaper or stronger execution path; Focusa should borrow the adaptive routing idea for bloat risk, not deletion authority.

## 3. Goals

- Keep hot-path prompts and Focus Slices summary-first and bounded.
- Keep docs navigable by separating canonical current docs from archival/history/spec worksheets.
- Keep proof automation centralized through generated or mapped proof bundles.
- Prefer Rust/core logic for packet building, filtering, scoring, and proof mapping.
- Make bloat visible early through advisory reports before fail-closed enforcement.

## 4. Non-goals

- No deletion or archival automation in v0.
- No blocking existing CI until a measured baseline and allowlist exist.
- No ban on shell scripts; new shell scripts require justification and should remain thin runners.
- No raw evidence loss; raw material may exist behind handles, artifacts, or explicit cold opt-in.

## 4.5 Relationship to Spec100 Context Cognition

Spec100 and Spec101 are complementary:

- Spec100 Context Cognition selects, structures, and renders bounded advisory context packets.
- Spec101 Focusa Bloatgaurd sets budgets, profiles, routines, tokenbloat controls, dead-code safety, and enforcement policy around those packets and related project surfaces.

Boundary:
- Bloatgaurd does not replace `ContextCognitionPacket`.
- Bloatgaurd's Context Compiler / Librarian should delegate scoped context selection to Spec100 Context Cognition where available.
- Bloatgaurd may add budget findings, profile choices, routine triggers, dedupe hints, and policy decisions around Spec100 packets.
- Context Cognition remains advisory and does not become canonical authority through Bloatgaurd.
- Workpoint, Trajectory, Evidence, ProjectIdentity, and operator steering keep the authority roles defined in Spec98-Spec100.

Shared vocabulary:
- selected context
- excluded context and exclusion reasons
- evidence refs
- rehydrate refs
- token budget
- advisory/canonical/degraded/stale envelope fields
- compact render and `--full` cold opt-in

## 5. Budget domains

### 5.1 Output firewall

Default route/tool responses use compact envelopes.

Initial checks:
- compact summary present
- evidence/ref handles present when larger data exists
- raw/full payload path gated by explicit opt-in
- line/byte/item caps documented for hot routes

Potential findings:
- route_without_compact_envelope
- ungated_full_payload
- hot_path_unbounded_output

### 5.2 Tool-call compression

Multi-step inspection should collapse into focused queries/scripts/route helpers when repeated.

Initial checks:
- repeated read/rg workflows have a candidate aggregate helper
- proof commands return summaries and handles, not raw logs
- inspection helpers expose `--json` or bounded text mode

Potential findings:
- repeated_manual_probe_pattern
- proof_output_too_verbose
- missing_summary_mode

### 5.3 Docs diet

Docs favor canonical short current pages, worksheets for structured contracts, and archive/history for long-lived narrative.

Initial checks:
- doc size budget by directory/class
- repeated command blocks detected
- current docs link to generated proof bundle map instead of duplicating long suites
- historical material marked archive/spec/worksheet

Potential findings:
- oversized_current_doc
- duplicated_proof_commands
- stale_history_in_current_doc

### 5.4 Test diet

Tests prefer Rust unit/integration or generated/static checks where appropriate. Shell suites remain thin orchestrators.

Initial checks:
- shell suite length budget
- new `.sh` files require comment/header justification
- static tests generated from contract maps where feasible
- one proof bundle map owns release proof command lists

Potential findings:
- shell_suite_too_large
- shell_script_without_justification
- duplicated_test_command_list

### 5.5 Context cache, not context dump

Prompts receive handle + summary + why relevant. Raw transcript/tool output is not durable authority.

Initial checks:
- Focus Slice section caps
- evidence refs instead of raw blobs
- full lineage/ontology/telemetry gated behind traversal/cold opt-in

Potential findings:
- raw_context_dump
- transcript_tail_authority
- uncapped_focus_slice_section

### 5.6 Rust-first core

Core bloat-control logic should migrate toward Rust crates where stable.

Initial checks:
- packet building/filtering/scoring/proof mapping implemented in Rust or has migration note
- JS/Pi/CLI surfaces remain adapters where practical
- new scripts do not duplicate core logic

Potential findings:
- adapter_contains_core_logic
- script_duplicates_core_logic
- rust_migration_candidate

### 5.7 Dead code and brownfield cleanup safety

Dead-code detection is part of the bloat budget, but deletion is never automatic. Focusa must distinguish dead code from temporarily unused, future-facing, compatibility, integration, test/migration, and dynamically referenced code.

Initial checks:
- import/export and call graph references
- API route registration and string-based dispatch
- CLI command registration and shell/script entry points
- Pi/MCP/tool contract registry references
- generated registries and docs/proof bundle references
- feature flags, env-gated behavior, degraded fallbacks, cron/background jobs, and external integration surfaces

Potential findings:
- unreferenced_script
- unregistered_route_handler
- unreachable_cli_command
- superseded_adapter
- stale_generated_entry
- zombie_doc
- duplicate_static_test
- obsolete_fallback_path

Evidence grades:
- A: unreachable by graph plus no route/CLI/tool/docs/proof/generated/external refs found
- B: superseded with a canonical replacement and migration evidence
- C: suspected only; needs runtime or integration verification
- D: protect; public, dynamic, future-facing, compatibility, or fallback surface

Deletion policy:
- Grade A may become a removal candidate after tests/proof bundle verification.
- Grade B should normally deprecate/isolate first, then remove in a later pass.
- Grade C is inventory only.
- Grade D is protected and should be documented or feature-flagged, not deleted.


### 5.8 Adaptive Bloatgaurd router/classifier

Focusa may use a NerfGuard-like classifier pattern to route cleanup and context work by bloat risk. This router is advisory for classification and deterministic for enforcement: it can recommend cleanup stage, gate mode, retrieval budget, and proof path, but it cannot authorize deletion or full-payload exposure by itself.

Intended routing decisions:
- compact summary vs cold full payload
- deterministic static scan vs LLM-assisted audit
- advisory report vs warning gate vs fail-closed gate
- docs split vs proof-map dedupe vs Rust/core migration vs script consolidation
- Grade A/B/C/D dead-code evidence path
- Focus Slice inclusion vs handle-only storage

Inputs:
- path/surface class: docs, scripts, tests, Rust route, Pi adapter, generated registry, evidence, fixture, prompt slice
- metrics: lines, bytes, duplicate command blocks, shell density, fixture size, output cap presence, full-payload flags
- authority hints: public API, CLI, Pi/MCP tool, UIAI integration, Workpoint/Trajectory/Focus State surface, fallback/recovery path
- evidence refs: call graph, route registry, proof bundle map, docs links, generated registry, tests

Outputs:
- `bloat_class`: output_firewall, docs_diet, test_diet, dead_code, context_dump, rust_migration, adaptive_route, other
- `risk`: low, medium, high
- `evidence_grade`: A, B, C, D when dead-code related
- `recommended_gate_mode`: advisory, warning, fail_candidate
- `recommended_stage`: 0 through 6 from the staged cleanup plan
- `required_verification`: proof bundle/tests/manual checks before action
- `allowed_actions`: report, document, isolate, deprecate, split, migrate, delete_candidate

Hard boundaries:
- Classifier output never deletes, archives, rewrites, or disables code.
- Classifier output never treats suspected dead code as safe removal without evidence grade and verification.
- Deterministic checks own CI failure decisions; LLM/classifier output can raise advisory findings only.
- Public/API/CLI/Pi/MCP/UIAI/generated/fallback surfaces default to protect unless evidence proves otherwise.
- Full payload exposure remains cold opt-in even when classifier says more context may help.

Potential findings:
- adaptive_route_missing
- classifier_overreach
- deterministic_gate_missing
- protected_surface_misclassified
- full_payload_recommended_without_cold_opt_in


### 5.9 Tokenbloat Control Domain

Tokenbloat Control is the prompt/runtime half of Focusa Bloatgaurd. It prevents growth in the model-visible context by combining query-aware compression, stable prefix caching, context handles, output firewalls, progressive disclosure, and duplicate-block dedupe.

Research inputs:
- LLMLingua / LongLLMLingua / LLMLingua-2: prompt and long-context compression, including query-aware compression and reordering.
- Repo Prompt / Context Builder: compile task-specific code/doc context from a repo graph instead of sending whole files or whole repos.
- Semantic caching and prompt caching: avoid repeated generation and preserve cacheable stable prefixes.
- KV/cache-aware layout: preserve byte-identical prefixes and isolate volatile state.
- Headroom-style gateway: optimize context before provider submission.
- Speculative/draft model routing: use a cheap classifier/compressor before strong-model reasoning.

Core architecture: stable-prefix + dynamic-slice prompts.

Stable prefix:
- provider/system/developer policy that rarely changes
- tool contract summaries in deterministic order
- project identity summary when verified and stable
- canonical instructions without timestamps, random IDs, or per-turn churn
- byte-identical serialization wherever possible to maximize provider prompt-cache hits

Dynamic slice:
- current ask
- active Workpoint packet summary
- trajectory gap/next action
- top relevant constraints/decisions
- evidence handles and rehydrate refs
- omitted counts and cold opt-in hints

Hard boundary:
- Dynamic slices carry volatile context; stable prefixes must not absorb per-turn evidence, generated timestamps, raw diagnostics, or transcript tail.

Mechanisms:

1. Query-aware context compression
   - Compress relative to current ask and active Workpoint gap.
   - Keep facts needed for the next action; drop unrelated docs/logs/history.
   - Candidate implementation surface: `focusa_traverse` + Workpoint/Trajectory gap → compact packet.

2. RepoPrompt-style Context Compiler
   - Build exact code/doc bundles from Spec100 Context Cognition packets, repo graph, symbols, docs maps, proof bundles, and active object refs.
   - Include files/symbols/snippets, not whole repo or broad docs trees.
   - Candidate Bloatgaurd mode: `context_compiler`; delegates candidate selection to Spec100 Context Curator where available.

3. LLMLingua-style prompt compression
   - Use heuristics or a small/local model to remove low-value tokens from long docs, logs, and prior summaries.
   - Compression remains advisory unless deterministic evidence proves safe omission.
   - Never compress safety/authority boundaries without explicit allowlist.

4. Semantic caching
   - Cache recurring summaries and answers by semantic similarity plus project_root/continuity_id scope.
   - Good targets: Workpoint resume summaries, trajectory views, project cards, repeated status/explain/next-step asks.
   - Cache hits return handle + summary + confidence, not raw cached transcript.

5. Prefix/prompt caching discipline
   - Stable prefix first; volatile content last.
   - Keep stable blocks byte-identical across turns when source facts have not changed.
   - Avoid timestamps, random IDs, unordered maps, and noisy counters in cacheable prefix.

6. KV/cache-aware prompt layout
   - Same discipline as prefix caching, with stronger serialization guarantees.
   - Deterministic order for tool contracts, AGENTS summaries, project identity, and static route guidance.
   - Dynamic slices appended after stable prefix to preserve provider-side reuse.

7. Context handles over context text
   - Store raw evidence/logs/docs/diagnostics in handles or indexed stores.
   - Prompt sees handle + summary + why relevant + rehydrate route.
   - Finding class: `raw_context_in_prompt` when raw blobs appear in hot path.

8. Output firewall
   - Tool outputs summarized before model-visible injection.
   - Compact envelope should include summary, top findings, evidence_refs, omitted_count, rehydrate_refs, next_tools.
   - Full payload remains cold opt-in.

9. Speculative/draft routing
   - Cheap model or deterministic classifier drafts compression/classification.
   - Strong model sees final compact packet.
   - Classifier is advisory; deterministic Bloatgaurd checks enforce.

10. Progressive disclosure
   - First response from index/summary.
   - Fetch targeted details only if required.
   - Route: summary → traverse slice → exact object → cold full payload.

11. Duplicate-block dedupe
   - Hash repeated docs, proof commands, logs, and boilerplate.
   - Replace repeats with `same_as:<hash>` plus canonical source ref.
   - Good targets: proof command lists, repeated release instructions, duplicated diagnostics blocks.

12. Conversation state distillation
   - Convert conversation history into structured state: decision, constraint, blocker, evidence, next action.
   - Drop prose history from hot context.
   - Bloatgaurd should measure leakage from raw transcript/prose into Focus Slice or reports.

Potential findings:
- stable_prefix_churn
- dynamic_slice_over_budget
- raw_context_in_prompt
- missing_rehydrate_ref
- duplicate_block_not_deduped
- semantic_cache_scope_missing
- context_compiler_missing_for_broad_read
- uncompressed_long_context
- cold_payload_used_by_default

Initial advisory metrics:
- stable prefix byte churn across adjacent turns
- dynamic slice token estimate
- raw blob byte count in prompt-visible surfaces
- duplicate block hashes in docs/proofs/logs
- cache hit/miss rates for project card, Workpoint, trajectory, and repeated status summaries
- ratio of handle refs to raw evidence bytes

Acceptance criteria before enforcement:
- define stable-prefix serialization boundaries
- define dynamic-slice section caps
- define semantic cache scope key: project_root + continuity_id + ask class + source hash
- define safe compression exclusions for safety, identity, authority, and operator directives
- add advisory report fields before any CI failure mode


### 5.10 Tool-call history elision and structured rehydration

Historical tool calls should not remain in the model-visible conversation as raw transcripts once their useful information has been distilled. Focusa Bloatgaurd should convert tool calls into structured summaries and rehydratable evidence handles, then remove or suppress raw tool-call payloads from hot prompts.

Core transform:

```text
ToolCallHistory → ToolRunSummary + EvidenceRef + RehydrateRef + Failure/Decision/Constraint links
```

Prompt-visible summary fields:
- tool or route name
- target object/path/endpoint
- action type: read, edit, test, search, diagnostics, proof, failure
- compact result: pass/fail/found/changed/no-op
- exact evidence handle
- omitted byte/line count when raw output was suppressed
- rehydrate route for exact raw output when needed
- linked decision/constraint/failure/workpoint when relevant

Lossless vs lossy split:
- Lossless raw tool output lives outside the hot prompt in artifact/evidence/tool-run storage.
- Lossy prompt summaries remain bounded and task-relevant.
- Exact raw output is available only through explicit rehydrate/cold opt-in.

Intelligence-preserving rules:
- Preserve failures, exact error classes, changed files, test names, command names, and recovery actions.
- Preserve active object refs and project_root/continuity_id scope.
- Preserve decisions/constraints produced from tool evidence.
- Do not summarize away exact diffs or error lines when they are the current blocker.
- Do not elide tool calls from the current active step until evidence capture/checkpoint succeeds.

Potential findings:
- raw_tool_history_in_hot_prompt
- tool_summary_missing_evidence_ref
- elided_failure_without_error_class
- unrecoverable_tool_output
- active_step_elided_too_early
- rehydrate_ref_missing

Default posture:
- Enabled for historical tool calls after checkpoint/evidence capture.
- Advisory-only for current-turn tool calls.
- Disabled for active blocker logs unless a summary includes exact error class and rehydrate ref.

## 6. Gate modes

### Mode A: advisory report

Default first rollout. Reports findings, exits 0.

### Mode B: warning gate

Prints findings and exits non-zero only for newly introduced high-confidence violations outside allowlist.

### Mode C: fail-closed gate

CI blocks violations after thresholds, baseline, and allowlist are reviewed.

## 7. Proposed initial thresholds

These are placeholders for measurement, not final policy.

| Domain | Advisory threshold | Fail candidate after baseline |
| --- | --- | --- |
| current doc size | > 400 lines or > 30 KB | > 600 lines or > 45 KB |
| shell script size | > 120 lines | > 200 lines without justification |
| shell suite size | > 180 lines | > 300 lines unless runner-only |
| fixture/tool output | > 50 KB | > 100 KB without compression/handle rationale |
| Focus Slice section | > configured section cap | uncapped section or raw dump |
| full payload flags | allowed only cold opt-in | hot-path/default full payload |

## 8. Allowlist model

Allowlist entries should include:

- path or route
- finding id
- reason
- owner/surface
- expiration or review trigger
- replacement/migration note when applicable

## 9. Report schema sketch

```json
{
  "schema": "focusa.bloat_budget_report.v0",
  "mode": "advisory",
  "summary": {
    "findings": 0,
    "high": 0,
    "medium": 0,
    "low": 0
  },
  "findings": [
    {
      "id": "oversized_current_doc",
      "severity": "medium",
      "path": "docs/current/example.md",
      "metric": "lines",
      "value": 650,
      "threshold": 400,
      "suggested_action": "split canonical/current content from archive/history"
    }
  ]
}
```

## 10. Rollout plan

1. Measure repo baseline in advisory mode.
2. Classify false positives and intentional exceptions.
3. Add allowlist with review dates.
4. Turn on warning gate for new violations.
5. Promote high-confidence checks to fail-closed mode.
6. Periodically reduce allowlist via Rust/core migrations and doc consolidation.

## 11. Open questions for iteration 1

- What line/byte thresholds should differ for specs, current docs, worksheets, generated files, and evidence files?
- Should proof command duplication be detected textually, from a generated proof map, or both?
- Which route/tool surfaces define the compact envelope contract canonically?
- Which shell scripts are permanent operational surfaces versus migration candidates?
- Should the gate live as a Rust CLI, Python static checker, or temporary shell wrapper around smaller checks?

## 12. Acceptance criteria for first implementation

Implementation should not start until this spec answers:

- baseline measurement command
- initial allowlist format
- advisory report schema
- exact checks for v1
- CI mode and failure policy

## 13. Focusa brownfield cleanup audit prompt v0

Use this prompt to produce an audit/spec, not to execute cleanup. It is intentionally conservative for Focusa's dynamic tools, route registries, generated docs, proof bundles, and future-facing Workpoint/Trajectory surfaces.

```text
You are a senior software architect and brownfield refactoring specialist.

Your task is to deeply analyze this project and identify dead code, unsafe complexity, outdated framework usage, duplicated logic, weak algorithms, and cleanup opportunities without breaking existing or planned features.

SPEC-FIRST RULE:
Produce an iterable audit spec first. Do not implement, delete, archive, rewrite, or refactor until the spec is reviewed and the operator explicitly authorizes execution.

FOCUSA-SPECIFIC AUTHORITY AND PROTECTION RULE:
Treat Workpoint, Trajectory, Focus State, Focus Slice, prediction, metacog, UIAI, MCP/Pi tool surfaces, proof-bundle surfaces, generated registries, CLI/API routes, degraded fallbacks, and docs-visible contracts as potentially public or integration surfaces even if statically unused.

Do not casually delete code. This is a brownfield project, so you must distinguish between:

1. Truly dead code
2. Temporarily unused code
3. Feature-flagged code
4. Underdeveloped future-facing code
5. Legacy compatibility code
6. Public API or integration surface code
7. Test-only or migration-only code
8. Code that appears unused but is dynamically referenced
9. Generated or registry-backed code
10. Degraded fallback or recovery-path code

Before recommending deletion, prove why the code is safe to remove.

Evidence grades:
- A = unreachable by import/call graph and no route/CLI/tool/docs/proof/generated/external refs found
- B = superseded with canonical replacement and migration evidence
- C = suspected only; needs verification
- D = protect; public, dynamic, future-facing, compatibility, fallback, or externally integrated

Classify each finding as one or more:
- dead_code
- bloat
- duplicated_proof_surface
- stale_docs
- obsolete_fallback
- rust_migration_candidate
- unsafe_complexity
- algorithm_or_architecture_opportunity

Work in multiple passes:

PASS 1 — Project map

Identify framework, runtime, build system, package manager, Rust crates, Node/Pi extension, CLI commands, daemon/API routes, MCP/Pi tools, generated registries, major directories, entry points, workers, cron/background jobs, tests, proof bundles, docs contracts, deployment assumptions, and external integrations.

Build a mental model of how the app works before suggesting changes.

PASS 2 — Usage tracing

Trace imports, exports, function calls, routes, components, hooks, services, config files, scripts, database models, generated files, docs links, proof bundle refs, CLI dispatch, MCP/Pi tool registration, API route registration, string-based dispatch, feature flags, environment-based behavior, event listeners, dynamic loading, plugin hooks, and external integrations.

Search for dynamic loading, reflection, string-based imports, framework conventions, plugin hooks, event listeners, route conventions, generated registries, environment behavior, and external clients before marking code dead.

PASS 3 — Dead code candidates

For each suspected dead code item, provide:

- File path
- Symbol/function/component/class/module/script/route name
- Finding class
- Evidence grade: A / B / C / D
- Why it appears unused
- Evidence refs
- Risk level: low / medium / high
- What could break if removed
- Whether it should be deleted, deprecated, isolated, documented, feature-flagged, migrated to Rust/core, or left alone
- Verification steps before removal

PASS 4 — Underdeveloped feature protection

Identify code that appears incomplete but likely represents future product direction.

Do not delete this code unless it is clearly abandoned. Recommend moving it behind feature flags, documenting it, adding ownership notes, isolating it from production paths, or linking it to a Workpoint/Trajectory/spec.

PASS 5 — Algorithm and architecture opportunities

Find opportunities to improve:

- Repeated logic
- Inefficient loops
- Expensive queries
- Over-fetching
- Poor state management
- Poor caching
- Large components/functions
- Unclear boundaries
- Tight coupling
- Framework anti-patterns
- Outdated APIs
- Security risks
- Error handling gaps
- Type safety gaps
- Test coverage gaps
- Adapter code duplicating Rust/core logic
- Raw context dumps instead of handles/summaries

For each opportunity, explain:

- Current problem
- Better approach
- Risk of changing it
- Suggested migration path
- Whether it belongs in this cleanup pass or a later refactor pass

PASS 6 — Framework/library upgrade opportunities

Identify outdated framework/library usage. Recommend modern equivalents only when stable and appropriate. Do not recommend upgrades just because they are newer. Explain compatibility risks, breaking changes, migration steps, and proof requirements.

PASS 7 — Safe execution plan

Create a staged cleanup plan:

Stage 0: baseline inventory and allowlist; no code changes
Stage 1: zero-risk docs/comments/registry cleanup
Stage 2: low-risk Grade A dead-code deletion with tests
Stage 3: Grade B deprecation/isolation behind verification
Stage 4: algorithm and architecture improvements
Stage 5: framework/library upgrades
Stage 6: deeper architectural cleanup and Rust/core migrations

For every stage include:

- Exact files/symbols affected
- Expected benefit
- Risk
- Required tests/proof bundles
- Rollback strategy

Rules:

- Do not delete or rewrite code until a full audit spec is produced and reviewed.
- Prefer small reversible commits after approval.
- Preserve behavior first; improve second.
- If uncertain, mark as needs verification, not dead.
- Treat tests, migrations, scripts, build config, env behavior, generated registries, docs contracts, proof bundles, and deployment scripts as first-class code.
- Assume some code may be used by external clients even if not referenced internally.
- Do not break public APIs, routes, database schemas, hooks, plugin contracts, MCP/Pi tools, UIAI integration, proof bundles, or recovery/degraded fallbacks.
- Produce exact recommendations, not vague advice.
- When proposing code changes later, show minimal diffs and explain why each change is safe.

Output format:

Brownfield Dead Code + Bloat Audit Spec

Project Map

Usage Tracing Notes

Dead Code Candidates

Keep / Protect / Future Feature Code

Bloat Budget Findings

Algorithm & Architecture Opportunities

Framework Upgrade Opportunities

Risk Matrix

Verification Plan

Staged Execution Plan

First Safe Spec Recommendations
```

## 14. NerfGuard applicability note

NerfGuard's public homepage is thin, but its installer suggests a real local gateway pattern: local proxy, CLI shims, hosted classifier endpoint, compression flag, upstream provider routing, and request fingerprint logging. The claim of up to 3x coding-agent usage is plausible when many requests are overpowered, but unproven without benchmark methodology and quality-regression data.

Focusa Bloatgaurd should borrow the pattern, not the trust model:

- Use adaptive classification to route bloat audits and context retrieval.
- Keep enforcement deterministic and evidence-backed.
- Keep deletion behind evidence grades, proof bundles, and operator authorization.
- Keep privacy-sensitive prompt/tool content in local handles where possible.
- Treat hosted or LLM classification as advisory only.

## 15. Configuration and toggle surface

Focusa Bloatgaurd is configurable by domain. Defaults should be conservative: summarize and warn first, fail only after baselines and allowlists are reviewed.

| Control | Default | Configurable values | Notes |
| --- | --- | --- | --- |
| `bloatgaurd.enabled` | true | true/false | Master advisory/report switch. |
| `bloatgaurd.mode` | advisory | advisory/warning/fail_closed | CI failure only after baseline/allowlist. |
| `bloatgaurd.output_firewall` | true | true/false/per-route | Compact envelopes by default. |
| `bloatgaurd.full_payload_policy` | cold_opt_in | cold_opt_in/allow/deny | Hot path should never default to full payload. |
| `bloatgaurd.context_compiler` | advisory | off/advisory/required_for_broad_reads | RepoPrompt-style exact bundle selection. |
| `bloatgaurd.query_aware_compression` | advisory | off/advisory/enforce_caps | Enforced mode caps dynamic slices; compression itself remains reviewable. |
| `bloatgaurd.semantic_cache` | scoped | off/scoped/global | Scoped key should include project_root + continuity_id. |
| `bloatgaurd.prompt_cache_layout` | on | on/off | Stable-prefix + dynamic-slice serialization. |
| `bloatgaurd.tool_history_elision` | after_checkpoint | off/advisory/after_checkpoint/aggressive | Aggressive requires strong rehydrate guarantees. |
| `bloatgaurd.dead_code_detection` | inventory | off/inventory/advisory/warning | Deletion never automatic. |
| `bloatgaurd.duplicate_dedupe` | advisory | off/advisory/warning | Hash repeated docs/proof/log blocks. |
| `bloatgaurd.shell_script_budget` | advisory | off/advisory/warning/fail_candidate | New scripts require justification before fail mode. |
| `bloatgaurd.docs_budget` | advisory | off/advisory/warning/fail_candidate | Thresholds vary by docs/current, worksheets, evidence, generated. |
| `bloatgaurd.adaptive_router` | advisory | off/advisory | Classifier cannot enforce deletion/full payload. |
| `bloatgaurd.speculative_draft_routing` | off | off/advisory | Cheap classifier/compressor before strong model. |
| `bloatgaurd.stable_prefix_churn_gate` | advisory | off/advisory/warning | Flags byte churn that harms provider caching. |

Per-surface overrides:
- docs: size thresholds, duplicate command thresholds, archive recommendations
- scripts/tests: line budgets, shell-density budgets, runner-only exemptions
- routes/tools: output caps, compact envelope requirement, full-payload opt-in
- prompts: stable-prefix boundaries, dynamic-slice caps, raw-tool-output policy
- evidence/logs: artifact storage path, summary schema, rehydrate retention
- dead code: evidence grade policy, allowlist, review expiration

Operator-facing switches should support:
- temporary disable for debugging
- strict mode for CI/release checks
- advisory mode for exploration
- cold opt-in for exact raw payloads
- protected-surface allowlist for public/API/CLI/Pi/MCP/UIAI/generated/fallback surfaces

Non-configurable safety rules:
- No automatic deletion.
- No classifier-only enforcement.
- No hidden full-payload default in hot paths.
- No cross-project semantic cache without project_root + continuity_id scope.
- No elision that drops active blocker evidence without a rehydrate ref.

## 16. Preconfigured profiles

Profiles tune context volume, compression, enforcement, and rehydration bias. They change model-visible context shape, not canonical truth. Workpoints, evidence stores, and rehydrate refs remain authoritative across profiles.

### 16.1 Daily Driver

Balanced default for ordinary development and operator sessions.

```yaml
profile: daily_driver
mode: advisory
output_firewall: true
tool_history_elision: after_checkpoint
context_compiler: advisory
semantic_cache: scoped
full_payload_policy: cold_opt_in
dynamic_slice_budget: medium
rehydration_bias: medium
```

### 16.2 Beast Mode

Max-intelligence profile for architecture, gnarly debugging, and high-uncertainty work.

```yaml
profile: beast_mode
mode: advisory
output_firewall: true
tool_history_elision: advisory
context_compiler: advisory
semantic_cache: scoped
full_payload_policy: cold_opt_in
dynamic_slice_budget: high
rehydration_bias: high
compression_bias: low
```

### 16.3 Speedy

Low-token fast path for routine coding, status, and small diffs.

```yaml
profile: speedy
mode: warning
output_firewall: true
tool_history_elision: after_checkpoint
context_compiler: required_for_broad_reads
semantic_cache: scoped
full_payload_policy: cold_opt_in
dynamic_slice_budget: low
stable_prefix_churn_gate: warning
rehydration_bias: low
```

### 16.4 Neat Freak

Audit/cleanup profile for Bloatgaurd, dead-code inventory, docs diet, script diet, and dedupe reviews.

```yaml
profile: neat_freak
mode: advisory
dead_code_detection: inventory
duplicate_dedupe: advisory
docs_budget: advisory
shell_script_budget: advisory
context_compiler: required_for_broad_reads
tool_history_elision: after_checkpoint
semantic_cache: scoped
full_payload_policy: cold_opt_in
```

### 16.5 Tightwad

Strict budget profile for CI/release gates and preventing new bloat.

```yaml
profile: tightwad
mode: fail_closed
output_firewall: true
full_payload_policy: deny_hot_path
duplicate_dedupe: warning
docs_budget: warning
shell_script_budget: warning
stable_prefix_churn_gate: warning
adaptive_router: advisory
tool_history_elision: after_checkpoint
```

## 17. Named routines and automation policy

Routines are operational workflows for Bloatgaurd. They can run manually or automatically depending on profile, project scope, and risk. Routine names should be memorable like profiles, while command names remain stable and scriptable.

### 17.1 The Patrol

Purpose: scan for docs, scripts, tests, route, prompt, tool-output, and dead-code bloat risks.

```yaml
routine: patrol
command: bloatgaurd scan
manual: true
automatic: ci_advisory | nightly | pre_pr
safe_default: report_only
recommended_profiles: [daily_driver, neat_freak, tightwad]
```

### 17.2 The Brief

Purpose: produce a bounded operator report with top findings, trend deltas, evidence refs, and next safe actions.

```yaml
routine: brief
command: bloatgaurd report
manual: true
automatic: daily | weekly | after_scan
safe_default: report_only
recommended_profiles: [daily_driver, neat_freak]
```

### 17.3 The Squeezer

Purpose: distill session/tool history into structured summaries, evidence refs, omitted counts, and rehydrate refs.

```yaml
routine: squeezer
command: bloatgaurd distill
manual: true
automatic: after_checkpoint | before_compaction | token_pressure_high
safe_default: no_raw_deletion
recommended_profiles: [daily_driver, speedy, tightwad]
```

### 17.4 The Librarian

Purpose: build RepoPrompt-style exact context bundles by delegating to Spec100 Context Cognition packets plus repo graph, docs maps, proof bundles, and active object refs.

```yaml
routine: librarian
command: bloatgaurd context
manual: true
automatic: broad_read_trigger | audit_start
safe_default: bounded_bundle_only
recommended_profiles: [beast_mode, neat_freak]
```

### 17.5 The Janitor

Purpose: detect duplicate docs/proof/log blocks and recommend canonical refs or `same_as:<hash>` replacements.

```yaml
routine: janitor
command: bloatgaurd dedupe
manual: true
automatic: ci_advisory | docs_changed
safe_default: advisory_only
recommended_profiles: [neat_freak, tightwad]
```

### 17.6 The Pantry

Purpose: warm or refresh scoped caches for project cards, Workpoint summaries, trajectory views, proof maps, and common status summaries.

```yaml
routine: pantry
command: bloatgaurd warm-cache
manual: true
automatic: project_open | docs_changed | after_checkpoint
safe_default: scoped_cache_only
recommended_profiles: [daily_driver, speedy]
```

### 17.7 The Gatekeeper

Purpose: enforce selected profile thresholds for CI/release/pre-commit policy checks.

```yaml
routine: gatekeeper
command: bloatgaurd check
manual: true
automatic: ci | pre_commit | release_gate
safe_default: profile_controls_failure_mode
recommended_profiles: [tightwad]
```

### 17.8 The X-Ray

Purpose: explain why a finding exists, why something was compressed, why something was protected, or why a full payload was denied.

```yaml
routine: xray
command: bloatgaurd explain
manual: true
automatic: false
safe_default: read_only
recommended_profiles: [daily_driver, beast_mode, neat_freak]
```

### 17.9 The Deep Dive

Purpose: rehydrate exact raw context for a specific handle when the bounded summary is insufficient.

```yaml
routine: deep_dive
command: bloatgaurd rehydrate
manual: true
automatic: active_blocker_only
safe_default: explicit_handle_required
recommended_profiles: [beast_mode]
```

### 17.10 The Scout

Purpose: advisory classifier/router that chooses likely bloat class, profile, routine, and verification path before expensive retrieval or audit work.

```yaml
routine: scout
command: bloatgaurd route
manual: true
automatic: before_broad_read | before_audit | token_pressure_high
safe_default: advisory_only
recommended_profiles: [daily_driver, speedy, neat_freak]
```

## 18. Routine automation matrix

| Routine | Manual | Automatic triggers | Writes code? | Can fail CI? | Cold payload allowed? |
| --- | --- | --- | --- | --- | --- |
| The Patrol | yes | CI advisory, nightly, pre-PR | no | profile-dependent | no |
| The Brief | yes | after scan, daily/weekly | no | no | no |
| The Squeezer | yes | checkpoint, compaction, token pressure | no | no | no |
| The Librarian | yes | broad read, audit start | no | no | bounded context only |
| The Janitor | yes | docs changed, CI advisory | no | warning/fail candidate only | no |
| The Pantry | yes | project open, docs changed, checkpoint | cache writes only | no | no |
| The Gatekeeper | yes | CI, pre-commit, release | no | yes | no |
| The X-Ray | yes | none | no | no | no |
| The Deep Dive | yes | active blocker only | no | no | yes, explicit handle only |
| The Scout | yes | broad read, audit, token pressure | no | no | no |

Automation rules:
- Automatic routines require verified `project_root + continuity_id` scope.
- Bloatgaurd context routines should call or consume Spec100 Context Cognition packets before inventing a separate context-selection path.
- Automatic routines emit report handles or evidence refs.
- Automatic routines do not delete, archive, rewrite, or disable code.
- Automatic routines do not expose full payloads except active-blocker Deep Dive with explicit handle policy.
- Manual routines can request cold opt-in, but must show why and what will be rehydrated.
- CI failure belongs to The Gatekeeper under Tightwad or explicitly selected strict modes.
- The Scout may recommend a routine/profile, but cannot enforce by itself.
