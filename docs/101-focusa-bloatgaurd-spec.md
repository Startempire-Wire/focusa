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


### 5.11 Optical context compression (pxpipe-inspired, default-safe)

Focusa Bloatgaurd may convert selected dense, non-verbatim-critical context into recoverable image artifacts when every safety gate passes. This is inspired by pxpipe’s observation that image-token cost is tied to pixel dimensions rather than raw character count, but Focusa MUST NOT adopt pxpipe’s exact assumptions: image render is lossy, model-dependent, and unsafe for exact identifiers. This domain is therefore default-on only as `safe_auto`, and the transform is a no-op unless every gate passes.

#### 5.11.1 Purpose

> Convert only dense, old, non-verbatim-critical context into recoverable image artifacts when provider policy, model capability, profitability, and readability gates all pass.

This domain lives beside the existing tokenbloat controls (query-aware compression, context compiler, LLMLingua-style compression, semantic caching, prefix caching, handles over raw context, output firewall, speculative routing, progressive disclosure, dedupe, and §5.10 tool-history elision). Bloatgaurd must reuse Spec 100 Context Cognition rather than create a competing context engine: Context Cognition may annotate compression hints, but Bloatgaurd decides transport and rendering.

#### 5.11.2 Integration point

After Context Cognition / Focus Slice / prompt packet render, before provider request forward, the pipeline is:

```text
Context Cognition / Focus Slice
        ↓
Bloatgaurd Context Decision
        ↓
Optical Compression Gate
        ↓
Provider Policy Gate
        ↓
Provider Request Injection
        ↓
Upstream Provider Forward
        ↓
Runtime Telemetry Capture
```

Component name: **Bloatgaurd Optical Context Gateway** (alias `Context Economizer`).

#### 5.11.3 Default-on posture (`safe_auto`)

```text
bloatgaurd.optical_context.enabled: safe_auto
bloatgaurd.optical_context.provider_policy_gate: required
bloatgaurd.optical_context.verified_models_only: true
bloatgaurd.optical_context.profitability_gate: required
bloatgaurd.optical_context.canary_gate: required
bloatgaurd.optical_context.keep_verbatim_text: true
bloatgaurd.optical_context.recoverable_store: required
bloatgaurd.optical_context.default_fallback: text_passthrough
bloatgaurd.optical_context.min_net_savings: 0.30
bloatgaurd.optical_context.max_quality_regression: 0
bloatgaurd.optical_context.full_payload_policy: cold_opt_in
```

Meaning: the feature is on by default, but the transform is a no-op unless every gate passes.

#### 5.11.4 What is imaged vs preserved

Imaged by default ONLY:

- old dense tool output
- old command logs
- old collapsed history after checkpoint
- large non-current tool docs
- large structured JSON already preserved behind a rehydrate ref
- diagnostic dumps where gist is enough

Never imaged (must remain verbatim text):

- current operator ask
- recent live turns
- Workpoint action authority
- Trajectory current goal/gap authority
- Evidence refs themselves
- secrets
- tokens
- hashes
- UUIDs
- 12-char identifiers
- file paths needed for edits
- exact diffs
- active error lines
- test names currently blocking work
- package versions involved in a fix
- security-sensitive content
- anything sparse/prose where image tokens do not win

#### 5.11.5 Provider Policy Ledger

```json
{
  "schema": "focusa.provider_policy_ledger.v1",
  "provider": "openai",
  "feature": "optical_context_compression",
  "status": "allowed | blocked | unknown | stale | needs_review",
  "official_policy_refs": [],
  "terms_hash": "sha256:...",
  "vision_docs_hash": "sha256:...",
  "checked_at": "2026-07-06T00:00:00Z",
  "expires_at": "2026-07-13T00:00:00Z",
  "review_required_on_change": true,
  "fallback": "text_passthrough"
}
```

Runtime rule: if `provider_policy_status != allowed`, do not image; use `text_passthrough`.

#### 5.11.6 Compatibility probe

```text
provider supports image input
provider counts image input normally as tokens
model accepts image input
model is Focusa-verified for dense text reading
pricing did not flip the profitability math
request limits still allow the payload
canary read passes
```

Any probe failure: `fallback=text_passthrough`, `reason=provider_policy_unknown | provider_banned | model_not_verified | image_rejected | canary_failed | not_profitable`.

#### 5.11.7 Strong fallback chain

1. Plain text ContextCognition render
2. Bloatgaurd compact envelope
3. Context handles + summaries + rehydrate refs
4. Tool-history elision after checkpoint
5. Semantic scoped cache
6. Deep Dive rehydrate for exact blocker evidence
7. No image transform (`text_passthrough`)

Raw source must never be destroyed. Every imaged block must carry:

```text
raw_ref
image_ref
rehydrate_ref
omitted_bytes
risk_class
provider_policy_ref
model_eval_ref
canary_status
fallback_used
```

#### 5.11.8 Forbidden optical context

`operator_current_ask`, `recent_turns`, `secrets`, `hashes`, `uuids`, `file_paths_needed_for_edit`, `exact diffs`. Optical compression MUST NOT silently cross these boundaries.

#### 5.11.9 Verification suite

```text
spec101_optical_context_defaults_static_test
provider_policy_gate_static_test
provider_terms_hash_change_fallback_test
image_input_rejected_fallback_test
model_allowlist_required_test
verbatim_guard_hash_uuid_secret_test
active_blocker_kept_text_test
profitability_gate_dense_vs_sparse_test
recoverable_ref_required_test
canary_failed_text_passthrough_test
context_cognition_no_canonical_mutation_test
focus_slice_no_raw_blob_default_test
```

These plug into the existing Spec 100 eval harness (prompt token waste, compaction recovery, precision/recall/F1, token budget savings, operator correction rate, packet render parity) rather than creating a separate proof universe.

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

### 5.12 Provider prompt cache + bounded recent-turns slice

Long-horizon Focusa sessions drain provider usage because every LLM call re-sends the full conversation context. Pi does not cache provider responses; `transport: "websocket-cached"` only caches the websocket connection. This domain introduces two related, additive levers to reduce per-turn token spend without sacrificing agent orientation: provider-side prompt caching and a bounded recent-turns slice. It sits alongside §5.10 (tool-history elision), §5.11 (optical context), and §10.4 (semantic caching) and reuses Spec 100 Context Cognition for the candidate set.

#### 5.12.1 Purpose

> Reduce provider token usage on long-horizon sessions by making the stable system prefix cacheable upstream and giving the agent a bounded, deduplicated orientation slice so it stops re-reading prior tool outputs and re-calling Workpoint/Trajectory surfaces.

When this domain is on:

- Provider sees a stable system prefix across consecutive turns and can cache it natively (Anthropic `cache_control: ephemeral` or OpenAI `prompt_cache_key`).
- Agent receives a recent-turns slice (default 4, hard cap 8) carrying only `[turn_id, mission_at_turn, outcome, evidence_refs]` per turn — never the assistant text.
- Pi-extension collapses the slice into the existing `## Cognitive Summary` section (no new section header) so prompt-token cost stays flat with current Focusa sessions.
- Status-only turns (`test`, `cont`, `ack`, etc. — matched via `isNonTaskStatusLikeText`) and tool-empty turns are filtered out before the slice is emitted.
- Hard guard: when `S.turnCount < 4`, no slice is emitted; when no qualifying turns exist, no slice is emitted.

#### 5.12.2 Integration point

After §5.11 optical compression and after §5.10 tool-history elision, before the provider request is forwarded:

```text
Context Cognition / Focus Slice
        ↓
Bloatgaurd Context Decision (§10.4 semantic cache lookup first)
        ↓
§5.10 Tool-history elision
        ↓
§5.11 Optical compression gate
        ↓
§5.12 Cacheable stable block split (this section)  ← NEW
        ↓
§5.12 Recent-turns slice emission (this section)   ← NEW
        ↓
Provider Request Injection (with cache_control breakpoint)
        ↓
Upstream Provider Forward
        ↓
Runtime Telemetry Capture (per-turn token in/out + cache_hit)
```

#### 5.12.3 Cacheable stable block + breakpoint

The pi-extension `before_agent_start` and `context` hooks (in `apps/pi-extension/src/turns.ts` and `…/compaction.ts`) currently mutate `event.systemPrompt` by appending plain strings. This domain introduces a structured split:

```ts
const stableBlock = [
  "## Focusa Cognitive Guidance",          // static rules
  "## Cognitive Summary",                  // intent/focus/decisions/constraints
  "## Workpoint Resume",                   // canonical mission + next_action + evidence
].join("\n\n");

const variableBlock = [
  "## Recent Turns (last 4)",              // NEW — bounded turn slice (deduped)
  "## Tool Result Tail",                   // active step tool output if any
].join("\n\n");

// Provider request body, in order:
{
  system: [
    { type: "text", text: stableBlock,   cache_control: { type: "ephemeral" } },
    { type: "text", text: variableBlock },
  ],
  // ...
}
```

Provider cache requirements:

- The stable block must contain **zero** variable content (no per-turn timestamps, no per-session paths, no per-tool-call output).
- The break must occur exactly once, between stable and variable.
- The pi-extension must support emitting `cache_control: ephemeral` (Anthropic) or equivalent on the stable block. When the underlying model SDK does not expose this, fall back to `safe_off` and emit a finding.

#### 5.12.4 Recent-turns ring buffer

State lives in pi-extension (`apps/pi-extension/src/state.ts`):

```ts
S.recentTurns: Array<{
  turn_id: string;           // mono increasing, e.g. `turn_<n>`
  mission_at_turn: string;   // active Frame mission or ask at turn start, bounded 120 chars
  outcome: string;           // bounded 80 chars: "committed | filed_bead | observed | blocked | ack"
  evidence_refs: string[];   // handles captured during this turn
  tool_call_count: number;
}> = [];
```

Ring buffer invariants:

- Hard cap: 8 entries. On overflow, oldest entry is dropped.
- Emission cap: default 4 (configurable via `bloatgaurd.recent_turns.n`; cold ceiling 8).
- Populated in the existing `turn_end` hook (`turns.ts:1111`), filtered by:
  - `!isNonTaskStatusLikeText(text)` (already exists in `compaction.ts`)
  - `tool_call_count > 0 || outcome !== "ack"`
- Empty array is a no-op; no empty `## Recent Turns` header is emitted.

#### 5.12.5 Trigger wiring

`event.systemPrompt += formatRecentTurnsSection()` is invoked in exactly three existing hook bodies (no new hooks):

| Hook | File:line | When |
|---|---|---|
| `before_agent_start` | `turns.ts:447` | every agent loop start (existing — keeps agent re-orientable after compaction) |
| `session_compact` | `compaction.ts:521` | after compaction (new injection on the resumed loop) |
| `model_select` | `turns.ts:1345` | after model switch (new model sees prior turns) |

Idempotency guard:

```ts
if (S.lastRecentTurnsSliceTurn !== S.turnCount) {
  S.lastRecentTurnsSliceTurn = S.turnCount;
  event.systemPrompt += formatRecentTurnsSection(4);
}
```

This guarantees one slice per turn across rapid agent loops, while still refreshing on compaction / model_select events that bump `S.turnCount`.

#### 5.12.6 Gate posture

Default posture (`bloatgaurd.recent_turns`):

- `enabled`: `safe_auto`
- `n_default`: 4
- `n_cold_max`: 8
- `drop_status_only`: true
- `drop_tool_empty`: true
- `fold_into_cognitive_summary`: true   (omit `## Recent Turns` header; render as sub-bullets)
- `cache_control_emitter`: `auto`       (auto-detect Anthropic / OpenAI SDK support)

Modes A → B → C graduated by per-turn token delta vs baseline.

#### 5.12.7 Initial checks

- Cacheable stable block split exists in `before_agent_start` and `context` hooks.
- `S.recentTurns` ring buffer is populated in `turn_end` and capped at 8.
- Recent-turns slice folds into existing `## Cognitive Summary`; no new top-level section.
- `cache_control: ephemeral` (or equivalent) emitted on stable block when supported.
- Provider cache hit reported by upstream is recorded in spec 29 telemetry.
- Idempotency guard prevents double-slice across rapid agent loops.

#### 5.12.8 Potential findings

- `provider_cache_breakpoint_missing` — stable block not split, no cache hint emitted.
- `uncapped_recent_turns_slice` — slice exceeds `n_cold_max` (8) or contains raw assistant text.
- `context_cache_dump_replacing_handle` — recent-turns slice inlines evidence payloads instead of using ECS handles.
- `redundant_injection_after_compaction` — slice re-emitted on every agent loop without idempotency guard.
- `raw_tool_history_replayed_in_slice` — turn slice includes tool output tail instead of evidence ref.
- `provider_cache_skipped_in_safe_auto` — fallback silently degrades without diagnostic.

#### 5.12.9 Default posture

- Enabled in `safe_auto` mode by default.
- Disabled entirely if upstream provider does not advertise prompt cache support and no SDK path is found; emits a single `provider_cache_skipped_in_safe_auto` finding on first turn.
- Recent-turns slice always enabled (separate from cache path) — it costs ~400-600 tokens per turn but prevents 2-5k token re-reads on the agent side, so net negative in typical sessions.

#### 5.12.10 Operator recall-intent trigger (last safeguard)

Detect operator phrases matching the recall-intent word set (see `docs/focusa-tools/recall-intent-words.md`) in the agent input handler, force-emit the recent-turns slice, and emit `recall_intent_triggered` telemetry with the matched category. When the daemon ring buffer is empty or fully filtered, surface `focusa_lineage_tree` and `focusa_awareness_packet` as alternative recall tools in the next-step affordances.

Trigger categories (high → low precision):

- direct recall: `recall`, `remember`, `remind me`, `bring me back`, `catch up`, `orient me`, `refocus`, `rewind`
- implicit prior: `what did we`, `earlier`, `last time`, `previously`, `where were we`, `as we discussed`, `you mentioned`, `you said`, `I asked`
- coherence loss: `context`, `on track`, `lost`, `confused`, `where were we going`, `what's the state`
- repetition: `again`, `already`, `already covered`, `duplicate`, `going in circles`
- operator steering: `wait`, `hold on`, `back up`, `scratch that`

False-positive guards:

- `again` matches recall only when no imperative mood is present (`why again?` = recall; `do X again` = not).
- `context` matches recall only when the operator phrase is ≤6 words.
- `I said` / `you said` are high-precision — always trigger.

Telemetry shape:

```json
{
  "event_type": "recall_intent_triggered",
  "matched_category": "implicit_prior",
  "matched_phrase": "earlier",
  "slice_size": 4,
  "ring_size": 7,
  "forced_re_emit": true,
  "alternative_tools_surfaced": []
}
```

#### 5.12.11 Adapter contract (cross-agent)

The recent-turns slice, recall-intent trigger, and cacheable-split features are delivered via a single adapter contract implemented once per agent (Pi, Claude Code, Aider, Cursor, Cline, Gemini CLI, etc.). The daemon owns the canonical ring buffer; adapters are thin clients.

Routes (all part of `focusa.recent_turns.v1`):

```text
GET  /v1/turns/recent?n=4&continuity_id=...   → RecentTurnsResponse
POST /v1/turns/recent                         → AppendTurnRequest (idempotent on turn_id)
POST /v1/events/recall-trigger                → telemetry ack
```

Canonical types (defined in `crates/focusa-core/src/recent_turns.rs`, mirrored in `apps/pi-extension/src/state.ts` for TS adapters):

```rust
pub struct AppendTurnRequest {
    pub turn_id: String,
    pub continuity_id: String,
    pub mission_at_turn: String,        // bounded 120 chars
    pub outcome: String,                 // committed|filed_bead|observed|blocked|ack|tooled
    pub evidence_refs: Vec<String>,
    pub tool_call_count: u32,
    pub emitted_at: u64,                 // unix seconds
}

pub struct RecentTurnSlice {
    pub turn_id: String,
    pub mission_at_turn: String,
    pub outcome: String,
    pub evidence_refs: Vec<String>,
    pub tool_call_count: u32,
    pub emitted_at: u64,
}

pub struct RecentTurnsResponse {
    pub schema: String,                  // "focusa.recent_turns.v1"
    pub count: usize,
    pub turns: Vec<RecentTurnSlice>,     // newest first
    pub fetched_at: u64,
}
```

Adapter responsibilities:

1. **Capture**: on turn_end (or agent-equivalent lifecycle event), POST `AppendTurnRequest` to the daemon with the current continuity_id.
2. **Inject**: on agent_start / session_compact / model_select (or equivalents), GET `/v1/turns/recent?n=4&continuity_id=...` and inject the formatted slice into the agent's system context.
3. **Trigger**: on operator input, run recall-intent detection against the canonical word set. On match, force-emit (reset idempotency guard), inject a 1-line nudge, and POST `recall-trigger` telemetry.
4. **Cache split**: at injection time, use `splitCacheableSystemPrompt` to mark the stable block; emit `cache_control` hint when the agent SDK supports it.
5. **Fail soft**: when the daemon is unreachable, the adapter skips injection silently and emits a `daemon_unavailable` telemetry event — never blocks the agent loop.

Adapters MUST NOT mutate the canonical ring buffer outside the documented routes. All ring mutations go through the daemon.

Default agent priority (which agent the operator can flag for primary):

- Pi: focusa-pi-extension adapter (this bead)
- Claude Code: focusa-claude-code-recent-turns-adapter (filed)
- Aider: focusa-aider-recent-turns-adapter (filed)
- Cursor: focusa-cursor-recent-turns-adapter (filed)
- Cline/Roo: focusa-cline-recent-turns-adapter (filed)
- Gemini CLI: focusa-gemini-recent-turns-adapter (filed)

Each adapter is a separate bead implementing the same contract.

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
