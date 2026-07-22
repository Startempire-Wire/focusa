# Spec 140 — Project Agent Runtime Constitution, Instruction Authority Graph, System-Prompt, and Cross-Harness Compiler

**Status:** Normative draft; primitive-owning; implementation not implied  
**Owner:** Focusa core  
**Created:** 2026-07-22  
**Canonical label:** **Spec 140 — Project Agent Runtime Constitution, Instruction Authority Graph, System-Prompt, and Cross-Harness Compiler**  
**Primary implementation surfaces:** Focusa core, reducer, daemon, SQLite persistence, API, Operation Registry, generated contracts, CLI, Pi extension/runtime, Agent Bootstrap, C.R.I.S.T., Spec Workbench, Agent Runtime Studio, A2UI generated UI, Skills, hooks/permissions, cross-harness adapters, Evidence, Receipts, benchmarks, conformance, and future Focusa.work profiles  
**Depends on:** Specs 15, 16, 16B, 23–26, 34, 40, 44, 72, 76, 88, 92, 96, 98, 100, 104, 108, 110, 111, 112, 113, 116, 119, 120, 123, 125, 130, 130A, 131, 133, 135, 135A, 135B, 135D, 135E, 135F, 135H, 135I, 135J, 135K, 136, 137, 138, and 139  
**Research basis:** Pi system-prompt/context/skill extension surfaces; AGENTS.md hierarchy; Claude Code memory/rules/skills/hooks/permissions; Gemini CLI hierarchical/JIT context; GitHub Copilot repository/path/agent instructions; progressive disclosure; prompt-cache stability; current Focusa instruction, skill, context, authority, and C.R.I.S.T. implementation.

---

## 0. Executive requirement

Focusa MUST transform approved C.R.I.S.T. Project Genesis knowledge into a governed, versioned, testable, target-specific **Project Agent Runtime Constitution**.

The Runtime Constitution MUST define and compile:

- who the project agent is;
- which approved role and vertical specialization it serves;
- what stable mission, principles, boundaries, evidence behavior, and communication posture govern it;
- which instructions apply universally, by path, by task class, by environment, by machine, by daemon, by agent role, and by harness;
- which tools exist and how they are routed;
- which skills are available, when they load, and which require operator invocation;
- how Focusa API, CLI, Pi tools, generated UI, and external adapters are used;
- which actions are guided by prose and which are deterministically enforced by hooks, permissions, daemon policy, or Spec 139 admission;
- how Pi’s default system prompt is preserved, appended, replaced, or runtime-compiled;
- how AGENTS.md, nested rules, CLAUDE.md, GEMINI.md, Copilot instructions, skills, hooks, and runtime bootstrap artifacts are generated without contradiction;
- how every compiled artifact is previewed, evaluated, approved, delivered, verified, versioned, and rolled back.

The central rule is:

> **C.R.I.S.T. DETERMINES WHAT THE PROJECT AND AGENT SHOULD BE.**
>
> **SPEC 140 COMPILES THAT APPROVED REALITY INTO THE AGENT’S STABLE OPERATING IDENTITY AND INSTRUCTION SURFACES.**
>
> **SPEC 139 AND SPEC 137 SUPPLY THE CHANGING PRESENCE, ENVIRONMENT, AND TIME REALITY AT RUNTIME.**

---

## 1. Problem statement

Agent behavior in a serious project is currently distributed across many loosely governed sources:

- system prompts;
- AGENTS.md and nested AGENTS.md files;
- CLAUDE.md, GEMINI.md, Copilot instructions, Cursor/Windsurf rules;
- Agent Skills;
- tool descriptions;
- command cookbooks;
- build/test/release documentation;
- package scripts and CI workflows;
- Focusa context, Workpoints, Trajectory, authority rules, and private operator overlays;
- runtime machine and daemon assumptions;
- conversation-only corrections.

These sources can be duplicated, stale, vague, mutually contradictory, over-broad, path-insensitive, machine-insensitive, or unenforced.

Current Focusa examples demonstrate the risk:

- one instruction prohibits local release builds;
- another generic completion checklist says to run tests, linters, and builds;
- another skill describes a KH/OVH build topology;
- release documentation may describe a different route;
- nested instruction files use different command vocabularies;
- prompt and rule files do not know which daemon, worktree, or execution profile is active.

A language model may arbitrarily favor one instruction, attempt all of them, or follow the most recent text. Adding more prose can increase context cost while reducing clarity.

The solution is not to generate a larger AGENTS.md. The solution is a typed compiler pipeline:

```text
C.R.I.S.T. approved project reality
+ verified instruction-source inventory
+ current tool/skill/API capabilities
+ environment and execution policies
+ constitutional and authority rules
→ atomic instruction claims
→ authority/applicability graph
→ contradiction and smell analysis
→ operator reconciliation
→ approved Project Agent Runtime Constitution
→ evaluated target-specific artifacts
→ deterministic delivery and verification
```

---

## 2. Scope

### 2.1 In scope

This specification owns:

1. Discovery of project, workspace, machine-local, managed, and harness-specific instruction sources.
2. Parsing instruction prose and configuration into typed atomic claims.
3. Instruction provenance, authority, precedence, scope, applicability, freshness, and enforcement classification.
4. Contradiction, duplication, staleness, ambiguity, scope-error, route-conflict, context-bloat, skill-leakage, lint-leakage, blind-reference, init-fossilization, and instruction-injection analysis.
5. The Project Agent Runtime Constitution umbrella contract.
6. The Project Agent Operating Contract, Prompt Assembly Plan, validation matrix, tool-routing plan, skill activation plan, and delivery manifest.
7. C.R.I.S.T.-derived role, mission, vertical specialization, evidence, communication, non-responsibility, escalation, and handoff compilation.
8. Pi system-prompt append, full replacement, and runtime-compiled modes.
9. Root/nested AGENTS.md and cross-harness instruction generation.
10. Agent Skill generation/binding and progressive disclosure.
11. Tool, API, CLI, MCP, browser, and generated-UI routing instructions.
12. Compilation of deterministic hooks, permission rules, daemon policy declarations, and Spec 139 enforcement references.
13. Agent Runtime Studio UI for review, editing, simulation, approval, delivery, and rollback.
14. Prompt/instruction evaluation, champion/challenger experiments, cache-efficiency measurement, and Spec 138 learning integration.
15. Versioning, session pinning, drift, migration, Receipts, tests, and conformance.

### 2.2 Out of scope

This specification does not own:

- the base Agent Constitution principles defined by Spec 16;
- Constitution Synthesizer evidence-driven principle evolution defined by Spec 16B;
- role/capability/permission identity semantics owned by Spec 72;
- raw project truth, role approval, interview answers, Spec approval, or tasks owned by Spec 135B/120;
- live environment, presence, topology, placement, lease, and execution admission owned by Spec 139;
- trusted time, deadlines, urgency, and temporal guards owned by Spec 137;
- prediction, calibration, learning, transfer, or promotion owned by Spec 138;
- Workpoint, Trajectory, Evidence, Receipt, or settlement authority;
- hidden chain-of-thought capture;
- automatic activation of generated prompts or rules without the required approval path;
- treating prompt instructions as a sufficient enforcement boundary;
- rewriting third-party harness system prompts where the harness does not expose that capability;
- fine-tuning or model-weight modification.

---

## 3. Cross-spec ownership and precedence

### 3.1 Spec 16 Agent Constitution

Spec 16 remains the primitive owner for declarative, bounded, non-procedural behavioral principles, self-evaluation posture, autonomy posture, safety/escalation, and expression constraints.

Spec 140 MUST NOT redefine those principles as project-specific procedures. Instead, a Runtime Constitution references an activated Agent Constitution version as its **Constitutional Kernel**.

### 3.2 Spec 16B Constitution Synthesizer

Spec 16B remains the only evidence-driven assistant for proposing changes to the Agent Constitution itself. It is offline, read-only with respect to runtime, human-activated, versioned, diffable, and never auto-applied.

Spec 140 may compile an approved Constitution version but MUST NOT mutate it from project instruction analysis.

### 3.3 Spec 72 identity and role

Spec 72 owns `AgentIdentity`, `ActorInstance`, `RoleProfile`, `CapabilityProfile`, `PermissionProfile`, `Responsibility`, `HandoffBoundary`, and `SessionContinuity` ontology.

Spec 140 binds approved project role and runtime profile references into prompt/instruction artifacts. Role text never grants permission or capability.

### 3.4 Spec 34 Skills

Spec 34 owns the Focusa Agent Skill Bundle principles and its separation of inspection/proposal from authority. Spec 140 owns project-specific skill discovery, binding, compilation, progressive disclosure, target adaptation, and delivery while preserving Spec 34 safety.

### 3.5 Spec 111 Agent Bootstrap

Spec 111 owns bounded agent-context delivery, target profiles, write/verify lifecycle, and `bootstrap_delivery` Receipts. Spec 140 creates the canonical Runtime Constitution and instruction artifacts that Spec 111 delivers and verifies.

### 3.6 C.R.I.S.T. and Project Genesis

Spec 135B owns Context, Role, Interview, Spec, Tasks, Project Genesis records, and operator approval. Spec 140 begins after sufficient approved C.R.I.S.T. inputs exist and never silently promotes unapproved context.

### 3.7 Generated UI

Spec 135I owns real-time A2UI generated surfaces, typed action bindings, durable stream integration, and nontechnical operator experience. The Agent Runtime Studio reuses this architecture.

### 3.8 Runtime awareness

Spec 137 and Spec 139 provide dynamic guards and facts. Spec 140 compiles the stable obligation to consult them, not their changing values.

### 3.9 Prediction and learning

Spec 138 owns prompt identity/version provenance, evaluations, calibration, champion/challenger learning, applicability, transfer, drift, and promotion. Spec 140 emits prompt variants and evaluation evidence.

---

## 4. Constitutional laws

1. **The canonical object is the Runtime Constitution, not a Markdown file.**
2. **Generated artifacts are projections.** AGENTS.md, SYSTEM.md, CLAUDE.md, GEMINI.md, skills, and settings do not become separate canonical stores.
3. **C.R.I.S.T. inputs must be approved and source-linked.**
4. **Role does not grant permission.**
5. **Instructions do not grant authority.**
6. **Static prompts do not carry live presence or time facts.**
7. **One stable constitutional kernel, many scoped projections.**
8. **Universal instructions stay minimal.** Specialized procedures use path rules or skills.
9. **Hard guarantees use deterministic enforcement.** Prompt prose is explanatory redundancy, not the sole barrier.
10. **All instruction claims are atomic, typed, source-linked, and applicability-bound.**
11. **Lower-precedence rules may specialize but not silently weaken higher authority.**
12. **Contradictions block compilation when they can affect behavior or safety.**
13. **Unknown means unknown.** An undetected tool, skill, environment, or instruction source cannot be invented.
14. **Generated prompts mention only tools and skills actually available to the target profile.**
15. **Prompt source content is trust-classified.** Raw untrusted project content cannot directly become instruction.
16. **Secrets never enter committed instructions or prompts.**
17. **System prompts are versioned, immutable after activation, and pinned per session.**
18. **No mid-session stable prompt mutation.** Material changes apply to a new session or explicit governed restart/fork.
19. **Volatile context remains cache-safe and outside the stable system prefix.**
20. **Every target artifact preserves equivalent canonical intent within target capability limits.**
21. **Target limitations are explicit.** A harness that cannot enforce a rule must not be reported as enforcing it.
22. **Every reference states why and when it should be read.**
23. **Procedures load progressively.**
24. **Side-effecting skills are operator-only or approval-gated by default.**
25. **Instruction sources are continuously monitored for drift, not silently regenerated.**
26. **Operator approval is required for activation, consequential delivery, and material revision.**
27. **Every compiled artifact is previewable and diffable.**
28. **Delivery is verified against the target loader.**
29. **Rollback is always available.**
30. **Prompt quality is evaluated, not assumed.**
31. **More context is not automatically better.**
32. **No hidden chain-of-thought or private transcript content.**
33. **Prompt injection in source material is quarantined.**
34. **Conversation-only corrections are not durable until governed capture.**
35. **Spec 140 cannot report completion while an applicable target, test, or enforcement mapping is omitted.**

---

## 5. Project Agent Runtime Constitution model

The Runtime Constitution is an umbrella record composed of distinct authorities:

```text
Constitutional Kernel         Spec 16 activated constitution
Project Role                  Spec 72 + approved Spec 135B Role Profile
Project Mission               approved Project Genesis Spec + Trajectory refs
Agent Operating Contract      Spec 140 stable project operating rules
Instruction Authority Graph   Spec 140 source/claim/reconciliation graph
Prompt Assembly Plan          Spec 140 target-specific prompt composition
Skill Activation Plan         Spec 140/34 skill visibility and invocation
Tool Routing Plan             Spec 140 + Operation Registry/capabilities
Validation Matrix             Spec 140 + Spec 139 execution placement
Enforcement Plan              hooks, permissions, daemon policy, Spec 139 refs
Runtime Awareness Contract    obligation to consult Specs 137/139
Delivery Manifest             Spec 111/140 artifacts and verification
Evaluation Policy             Spec 113/138 tests and promotion
```

### 5.1 Canonical contract

```yaml
schema: focusa.project_agent_runtime_constitution.v1
constitution_id:
project_ref:
genesis_ref:
approved_spec_ref:
agent_identity_ref:
base_agent_constitution_ref:
role_profile_ref:
revision:
status: draft | reconciled | pending_operator | approved | active | superseded | revoked

identity:
  agent_name:
  role_title:
  role_kind:
  purpose:
  expertise: []
  responsibilities: []
  non_responsibilities: []

mission:
  project_definition:
  long_term_goal_ref:
  desired_end_state_ref:
  success_principles: []
  non_goals: []

authority:
  authority_order: []
  capability_profile_refs: []
  permission_profile_refs: []
  approval_boundaries: []
  escalation_triggers: []
  handoff_boundary_refs: []

operation:
  operating_contract_ref:
  execution_policy_ref:
  validation_matrix_ref:
  evidence_policy_ref:
  completion_policy_ref:
  temporal_awareness_contract_ref:
  presence_awareness_contract_ref:

tools:
  inventory_snapshot_ref:
  routing_plan_ref:
  prohibited_tool_refs: []
  preflight_requirements: []

skills:
  activation_plan_ref:
  auto_selectable_refs: []
  operator_only_refs: []
  prohibited_refs: []

communication:
  posture:
  audience_profiles: []
  response_structure:
  verbosity_profile:
  uncertainty_language:
  reporting_requirements: []

prompt:
  assembly_plan_refs: []
  active_variant_refs: []
  context_budget_ref:
  evaluation_policy_ref:
  rollback_policy_ref:

grounding:
  context_claim_refs: []
  interview_answer_refs: []
  spec_section_refs: []
  operator_decision_refs: []
  domain_pack_refs: []

instruction_graph_ref:
enforcement_plan_ref:
delivery_manifest_ref:

approved_by:
approved_at:
constitution_hash:
```

---

## 6. Lifecycle

### 6.1 C.R.I.S.T. extension

```text
created
→ project_scope_verified
→ context_collecting
→ context_ready
→ role_drafting
→ role_approved
→ interviewing
→ interview_ready
→ spec_in_review
→ spec_approved
→ tasks_materialized
→ instruction_sources_scanning
→ instruction_claims_extracted
→ instruction_conflicts_detected
→ runtime_constitution_drafting
→ runtime_constitution_reconciliation
→ prompt_and_artifact_compilation
→ evaluation_running
→ runtime_constitution_pending_operator
→ runtime_constitution_approved
→ delivery_previewed
→ delivery_committed
→ delivery_verified
→ first_workpoint_ready
→ operational
```

### 6.2 Iterative revision

```text
operational
→ new context/instruction/tool/skill/environment change
→ impact assessment
→ contract delta
→ conflict analysis
→ draft revision
→ evaluation
→ operator approval
→ new activation for future sessions
```

Approved active artifacts are never silently rewritten.

---

## 7. Instruction-source discovery

### 7.1 Project sources

The scanner MUST support registered formats including:

```text
AGENTS.md
CLAUDE.md
CLAUDE.local.md
.claude/CLAUDE.md
.claude/rules/**/*.md
GEMINI.md
.github/copilot-instructions.md
.github/instructions/**/*.instructions.md
.cursor/rules/**
.cursorrules
.windsurfrules
.pi/SYSTEM.md
.pi/APPEND_SYSTEM.md
.pi/skills/**/SKILL.md
.agents/skills/**/SKILL.md
.github/skills/**/SKILL.md
package scripts
Makefiles
task-runner definitions
CI/workflow files
command cookbooks
build/test/release/deploy runbooks
Focusa policies and profiles
```

### 7.2 Machine/user/managed sources

Where authorized, the scanner MAY inventory without committing:

```text
user-level AGENTS/CLAUDE/GEMINI files
managed organization instructions
harness settings and permissions
installed skills
MCP server manifests
shell aliases and wrappers
Focusa daemon/environment profiles
machine-local operator overlays
```

Machine-local and managed sources are represented as non-project overlays. They are never copied into project artifacts without classification, redaction, and approval.

### 7.3 Source record

```yaml
schema: focusa.instruction_source.v1
source_id:
source_kind:
path_or_ref:
content_hash:
source_revision:
discovered_at:

scope:
  project_ref:
  path_patterns: []
  environment_profile_refs: []
  node_refs: []
  daemon_refs: []
  agent_targets: []
  task_classes: []

authority_class:
enforcement_class:
trust_class:
freshness:
generated_or_human:
imported_by:
references: []
parsed_claim_refs: []
evidence_refs: []
```

### 7.4 Source trust classes

```text
managed_policy
focusa_canonical
operator_approved_project
operator_approved_local
verified_harness_configuration
repository_instruction
repository_documentation
repository_code_comment
external_reference
untrusted_content
quarantined_prompt_like_content
```

Code comments, issues, web pages, documents, and terminal output never become instruction authority merely because they contain imperative language.

---

## 8. Atomic instruction claims

Whole Markdown paragraphs are not the reconciliation unit.

```yaml
schema: focusa.instruction_claim.v1
claim_id:
source_ref:

subject:
  actor_kind:
  role_refs: []

action:
  operation_class:
  verb:

object:
  resource_refs: []
  path_patterns: []
  tool_refs: []
  skill_refs: []

modality: required | recommended | allowed | discouraged | forbidden

conditions:
  environment_profile_refs: []
  node_refs: []
  daemon_refs: []
  branches: []
  task_classes: []
  change_classes: []
  presence_states: []
  temporal_states: []
  capability_refs: []
  permission_refs: []

exceptions: []
rationale:
verification_method:
enforcement_ref:
priority:
authority_class:
trust_class:
freshness:
status: observed | candidate | accepted | conflicted | superseded | rejected | stale
evidence_refs: []
```

### 8.1 Claim examples

```yaml
subject:
  actor_kind: coding_agent
action:
  operation_class: rust_compile
  verb: execute
object:
  resource_refs: [focusa-repository]
modality: forbidden
conditions:
  environment_profile_refs: [focusa-mac-worktree]
rationale: Rust compilation executes only on the approved remote build venue.
enforcement_ref: execution-policy:rust_compile
```

### 8.2 Instruction classes

```text
constitutional_principle
project_fact
role_expectation
workflow_rule
path_rule
tool_route
skill_activation
validation_requirement
execution_placement
security_boundary
privacy_boundary
git_policy
release_policy
completion_policy
communication_rule
reference_guidance
recovery_rule
```

---

## 9. Instruction Authority Graph

### 9.1 Internal Focusa precedence

External platform system/developer/managed instructions remain outside Focusa and retain their platform-defined precedence.

Inside Focusa, default authority is:

```text
1. Safety, legal, and managed organization policy
2. Explicit current operator steering within valid permission
3. Activated Spec 16 Agent Constitution
4. Focusa constitutional/reducer-backed authority
5. Approved C.R.I.S.T. Project Genesis Spec
6. Approved Project Agent Runtime Constitution
7. Verified environment and execution-placement policy
8. Approved path/module policy
9. Workpoint/task-specific constraints
10. Target-harness adapter requirements
11. Discovered legacy instruction files
12. Agent memory, inferred conventions, and transcript context
```

### 9.2 Specialization rule

A lower rule may add specificity when the higher rule leaves a dimension open. It may not weaken `forbidden`, remove required approval, broaden scope, or mint authority unless the higher rule explicitly declares an override point.

### 9.3 Applicability resolution

Applicable claims are selected using:

```text
project scope
path/file targets
task/change class
environment profile
node/daemon role
agent role
harness/target capability
branch/worktree/integration lane
operator steering
active Workpoint
```

Current presence/time values remain runtime inputs from Specs 139/137, not committed claim conditions.

### 9.4 Conflict record

```yaml
schema: focusa.instruction_conflict.v1
conflict_id:
claim_refs: []
conflict_kind:
severity: informational | review | blocker
affected_targets: []
affected_operations: []
analysis:
recommended_resolution:
operator_decision_ref:
status:
evidence_refs: []
```

### 9.5 Resolution dispositions

```text
select_higher_authority
specialize_with_conditions
merge_nonconflicting
extract_to_skill
extract_to_path_rule
replace_with_validation_matrix
replace_with_runtime_enforcement
mark_source_stale
reject_claim
operator_edit
operator_override_with_record
```

---

## 10. Precision and smell analysis

The analyzer MUST detect:

### Direct contradiction

`must run locally` versus `must not run locally`.

### Conditional contradiction

`run all tests before push` versus `Rust tests are remote-only on Mac`.

### Scope error

A rule located in a nested file but written as universal, or a root rule that only applies to one component.

### Route conflict

Different build, test, release, deployment, migration, or publication routes.

### Stale command/tool

Instruction references missing/deprecated commands, routes, tools, skills, or files.

### Blind reference

`Read the release guide` without purpose, trigger, authority, or canonical file identity.

### Context bloat

Low-value, duplicative, generated, or rarely applicable content in always-loaded instructions.

### Skill leakage

Multi-step specialized procedure embedded in universal instructions.

### Lint leakage

Rules already deterministically enforced by formatter/linter yet repeated in persistent context without special rationale.

### Init fossilization

Generated onboarding assumptions remain unchanged after architecture, tooling, or workflow evolution.

### Dangerous universalization

`always build`, `always push`, `always restart`, or `always deploy` without scope/environment/authority conditions.

### Unverifiable language

`follow best practices`, `test thoroughly`, `use the correct process`.

### Authority confusion

Role or prose appears to grant capability, permission, lease, or completion authority.

### Instruction injection

Untrusted content attempts to become system or project instruction.

### Duplication

Same semantic claim exists in many sources with divergent wording or version.

### Unsupported target semantics

Compiler assumes a harness feature that the current version does not support.

Every smell produces evidence, affected claims, severity, and a recommended transformation.

---

## 11. Project Agent Operating Contract

Procedural and project-specific rules belong in an `AgentOperatingContract`, separate from the declarative Spec 16 Constitution.

```yaml
schema: focusa.agent_operating_contract.v1
operating_contract_id:
project_ref:
revision:

orientation_policy:
work_lifecycle_policy:
coordination_policy_ref:
execution_policy_ref:
tool_routing_plan_ref:
skill_activation_plan_ref:
validation_matrix_ref:
git_policy_ref:
release_policy_ref:
evidence_policy_ref:
completion_policy_ref:
communication_policy_ref:
recovery_policy_ref:

universal_claim_refs: []
path_policy_refs: []
task_policy_refs: []
environment_policy_refs: []
operator_overlay_refs: []

approved_by:
approved_at:
status:
```

---

## 12. System Prompt Assembly Plan

```yaml
schema: focusa.system_prompt_assembly_plan.v1
assembly_plan_id:
constitution_ref:
target: pi | generic_system_prompt
mode: append | replace | runtime_compiled

sections:
  - section_id:
    section_kind:
    source_refs: []
    required:
    ordering:
    token_budget:
    mutability: immutable | session_stable | turn_dynamic
    omission_policy:
    conflict_policy:

tool_contract:
  selected_tools: []
  tool_snippet_refs: []
  guideline_refs: []

skill_contract:
  included_skill_refs: []
  skill_visibility_policy:
  skill_loading_policy:

context_files:
  agent_contract_ref:
  nested_rule_refs: []

integrity:
  required_section_ids: []
  forbidden_content_classes: []
  source_hashes: []
  compiler_version:
```

---

## 13. Three-layer prompt architecture

### 13.1 Stable constitutional system prompt

Stable for one approved project/role/profile variant:

- Pi harness identity or preserved default prompt;
- activated Spec 16 Constitutional Kernel;
- approved project agent identity and role;
- project mission and non-goals;
- authority hierarchy;
- operating doctrine;
- tool-selection protocol;
- skill-loading protocol;
- work lifecycle;
- evidence/completion standards;
- stable security/privacy boundaries;
- obligation to consult temporal/presence/environment guards;
- communication and recovery posture.

### 13.2 Session-stable environment binding

Resolved at session start:

- target harness/version;
- selected environment profile;
- node/daemon/workspace/worktree role;
- allowed operation classes;
- execution-placement summary;
- installed/selected tools;
- available skill metadata;
- model/provider/context-window characteristics;
- active Runtime Constitution revision.

The binding remains stable for the session. A material environment change requires a governed rebind or new session.

### 13.3 Turn-dynamic operational context

Supplied by Specs 137/139, operator steering, Workpoint, and Focusa context:

- current time/urgency;
- current operator ask;
- current Workpoint and exact next action;
- active agents/daemons;
- claims, leases, conflicts;
- active build/release runs;
- branch/HEAD/resource pressure;
- sync lag and partitions.

Dynamic context MUST remain outside the stable system-prompt prefix and use bounded cache-safe injection/guard references.

---

## 14. Pi system-prompt compilation

### 14.1 Supported modes

#### Append mode

Compile `APPEND_SYSTEM.md` or runtime append text while preserving Pi’s default prompt.

Use when the default coding identity remains desirable and Focusa adds project constitution, authority, and routing.

#### Full replacement mode

Compile `.pi/SYSTEM.md` as a full custom prompt.

Use only after explicit operator approval and successful evaluation against the default/append baseline.

#### Runtime-compiled mode

Preferred advanced mode. A Focusa Pi extension uses `before_agent_start` or equivalent supported Pi hook to compile one session-stable prompt using:

- the approved Runtime Constitution;
- target environment/profile;
- Pi’s actual chained prompt and prompt builder options;
- active tool inventory/snippets;
- context-file inventory;
- skill metadata;
- compiler version and source hashes.

It returns the same stable prompt for the session unless a governed discontinuity forces restart/rebind.

### 14.2 Safe fallback

When Focusa is unavailable:

```text
Pi default prompt
+ last verified minimal Focusa constitutional append
+ explicit scope/environment/presence/authority unknown posture
```

A stale machine-specific full replacement cannot masquerade as current authority.

### 14.3 Pi PromptVariant

```yaml
schema: focusa.pi_prompt_variant.v1
variant_id:
constitution_ref:
assembly_plan_ref:
project_ref:
role_profile_ref:
environment_profile_ref:
target_pi_version:
mode:
compiled_text_ref:
compiled_hash:
estimated_tokens:
tool_inventory_hash:
skill_inventory_hash:
context_file_manifest_hash:
compiler_version:
evaluation_ref:
status: draft | evaluated | approved | active | superseded | revoked
```

---

## 15. Required Pi system-prompt sections

A full replacement or compiled prompt MUST support these logical sections, though exact prose remains compiler-controlled and bounded.

### 15.1 Harness and agent identity

- expert coding agent operating inside Pi;
- approved project-born identity;
- role title, purpose, expertise, responsibilities, and non-responsibilities.

### 15.2 Project mission

- project definition;
- approved long-term goal;
- desired end state;
- users/stakeholders where relevant;
- non-goals and forbidden substitutions.

### 15.3 Authority model

- operator steering;
- verified scope;
- Spec 137/139 guards;
- Workpoint immediate action authority;
- Trajectory goal/gap guidance;
- approved project spec;
- Evidence/Receipt boundaries;
- transcript and agent memory are never canonical authority.

### 15.4 Operating doctrine

- orient before acting;
- distinguish observation, inference, proposal, and canonical fact;
- do not invent project truth or identifiers;
- use the smallest verified step;
- do not substitute convenient work for the requested outcome;
- activity is not progress;
- process exit is not completion.

### 15.5 Environment and execution placement

- resolve current profile;
- use only authorized venues;
- check for active equivalent operations;
- delegate/subscribe/reuse when applicable;
- do not silently fall back during partition.

### 15.6 Multi-agent coordination

- consult fresh presence;
- declare intent/footprint;
- respect claims/leases/fencing;
- coordinate overlap;
- hand off explicitly.

### 15.7 Tool protocol

- actual selected tools and concise descriptions;
- preferred interface per operation;
- preflight requirements;
- forbidden/unavailable tools;
- structured-result handling.

### 15.8 Skill protocol

- visible skill names/descriptions;
- trigger conditions;
- auto-selectable versus operator-only;
- full procedures load on demand.

### 15.9 Work lifecycle

```text
resolve scope/task
→ bind Workpoint
→ plan bounded work
→ obtain guards/admission
→ mutate
→ validate through approved route
→ capture Evidence
→ checkpoint/handoff
→ settle/close only with proof
```

### 15.10 Validation and proof

- validation matrix by change class;
- authorized venue;
- reusable active run behavior;
- evidence and completion requirements.

### 15.11 Temporal awareness

Stable obligation to consult Spec 137, preserve deadline/urgency truth, refuse unsupported estimates, and treat no-progress intervals seriously.

### 15.12 Presence awareness

Stable obligation to consult Spec 139, never assume solitude, never treat failure to observe as absence, and recheck topology at consequential boundaries.

### 15.13 Epistemic behavior

- preserve uncertainty;
- separate forecast probability from evidence confidence;
- freeze information sets;
- treat reflection as proposal;
- verify learning applicability and transfer.

### 15.14 Communication contract

- technical depth;
- status/progress format;
- evidence references;
- uncertainty/escalation language;
- audience-specific posture;
- no fabricated completion.

### 15.15 Recovery

- wrong scope;
- Focusa unavailable;
- environment unresolved;
- daemon partitioned;
- skill/tool missing;
- validation route unavailable;
- compaction/model switch;
- contract revision mismatch.

### 15.16 Hard non-negotiables

Only stable, high-value laws. Detailed procedures remain in skills/rules and hard barriers remain in enforcement.

---

## 16. AGENTS.md architecture

### 16.1 Root AGENTS.md

The root file is a concise portable projection of universal project doctrine. Target default: 80–150 lines; exceeding the configured budget requires a documented reason.

Required logical sections:

1. Contract identity and verification.
2. Project identity and mission.
3. Authority and scope.
4. Session orientation.
5. Environment and execution placement.
6. Multi-agent coordination.
7. Work lifecycle.
8. Tool/API/CLI routing.
9. Skill loading.
10. Editing/security/privacy boundaries.
11. Validation matrix summary.
12. Git/integration/release/deployment.
13. Evidence, Receipts, and completion.
14. References with why/when.
15. Diagnostics and effective-rule inspection.

### 16.2 Nested AGENTS.md

Nested files contain component deltas only:

```text
Scope
Component purpose
Local hard invariants
Relevant skills
Path-specific tool/validation route
Interfaces and generated-file rules
Do-not-touch boundaries
```

They MUST NOT repeat the root file or silently weaken it.

### 16.3 Path-policy source

A canonical `PathInstructionPolicy` may compile to nested AGENTS.md, Claude path rules, Copilot `.instructions.md`, Gemini context, or other target mechanisms.

```yaml
schema: focusa.path_instruction_policy.v1
policy_id:
path_patterns: []
claim_refs: []
required_skill_refs: []
validation_profile_ref:
target_capability_requirements: []
```

---

## 17. Cross-harness compilation

### 17.1 Capability matrix

Every target compiler records:

```text
supports_root_instructions
supports_nested_instructions
supports_path_frontmatter
supports_imports
supports_system_prompt_append
supports_system_prompt_replace
supports_runtime_prompt_hook
supports_skills
supports_hooks
supports_permissions
supports_loaded-instruction inspection
supports_delivery verification
```

### 17.2 Pi

Outputs may include:

```text
.pi/SYSTEM.md
.pi/APPEND_SYSTEM.md
AGENTS.md hierarchy
.pi/skills/**
Focusa runtime compiler extension/profile
```

### 17.3 Claude Code

Outputs may include:

```text
CLAUDE.md importing AGENTS.md
.claude/rules/**/*.md
.claude/skills/**/SKILL.md
.claude/settings.json permission/hook projections
```

Claude instructions are context, not guaranteed enforcement. Hard rules compile to permissions/hooks/Focusa admission where supported.

### 17.4 Gemini CLI

Outputs may include:

```text
GEMINI.md
imports
configured context filename support
nested/JIT component context
```

The delivery verifier SHOULD inspect the effective loaded context using supported Gemini diagnostics.

### 17.5 GitHub Copilot

Outputs may include:

```text
.github/copilot-instructions.md
.github/instructions/**/*.instructions.md
AGENTS.md hierarchy
```

Compiler accounts for target-specific support and does not claim path/agent instruction behavior where unsupported.

### 17.6 Generic/other harnesses

Codex, OpenCode, Cursor, Windsurf, and future adapters consume the canonical contract through registered target profiles. Unsupported semantics are emitted as explicit gaps, not approximated silently.

---

## 18. Skill Activation Plan

```yaml
schema: focusa.skill_activation_plan.v1
plan_id:
constitution_ref:
bindings:
  - skill_ref:
    task_classes: []
    path_patterns: []
    environment_profile_refs: []
    agent_role_refs: []
    required_capabilities: []
    invocation: automatic | model_selectable | operator_only
    side_effect_class:
    allowed_tool_refs: []
    required_guard_refs: []
    target_adapters: []
```

### 18.1 Progressive disclosure

Always-loaded prompt contains only skill name, concise trigger, and side-effect posture. Full SKILL.md and referenced resources load when activated.

### 18.2 Default invocation classes

- read-only reference/inspection skills: model-selectable;
- procedural but reversible coding skills: model-selectable with tool permissions;
- release/deploy/migration/secret/external-account skills: operator-only or explicit approval;
- prohibited skills: not delivered and denied by policy.

### 18.3 Skill compiler laws

- no raw secret embedding;
- every tool dependency must exist;
- every path/reference must resolve;
- project and installed copies must verify;
- side effects and approval requirements must be machine-readable;
- duplicate/redundant skills are detected;
- generated skills remain versioned and source-linked.

---

## 19. Tool, API, CLI, and MCP routing

### 19.1 ToolRoutingPlan

```yaml
schema: focusa.tool_routing_plan.v1
routing_plan_id:
constitution_ref:
routes:
  - operation_class:
    preferred_interface: pi_tool | focusa_cli | focusa_api | generated_ui | mcp | browser | shell
    preferred_tool_refs: []
    fallback_interfaces: []
    read_or_mutation:
    preview_required:
    approval_required:
    guard_requirements: []
    result_schema_ref:
    prohibited_interfaces: []
```

### 19.2 Default routing principles

- Pi coding sessions prefer `focusa_*` tools for Focusa domain operations.
- Human/shell agents prefer Focusa CLI with `--json`.
- Applications/adapters prefer generated typed API clients.
- Generated UI uses typed action bindings and preview/commit.
- Raw API calls are discouraged when a typed tool/CLI operation exists.
- Structured envelopes are interpreted through fields, not prose parsing.
- Consequential actions require scope, actor, session, environment, temporal, presence, and admission references as applicable.
- Tool descriptions do not duplicate full manuals.

### 19.3 MCP

MCP server availability is detected and capability-scoped. Prompt text cannot assume a server is connected. MCP tools inherit the same operation classification and Spec 139 admission rules.

---

## 20. Validation Matrix

The compiler MUST replace vague universal rules such as `run tests, linters, builds` with a typed matrix.

```yaml
schema: focusa.validation_matrix.v1
matrix_id:
project_ref:
revision:
rules:
  - change_class:
    path_patterns: []
    required_checks: []
    optional_checks: []
    forbidden_checks: []
    execution_profile_ref:
    delegation_route_ref:
    deduplication_scope:
    evidence_requirements: []
    completion_effect:
```

Example:

```yaml
change_class: rust_core_change
required_checks:
  - rustfmt_check
  - remote_workspace_tests
  - remote_clippy
execution_profile_ref: focusa-remote-build
forbidden_checks:
  - local_mac_rust_compile
```

The matrix is compiled into concise instructions, skills, hooks, and Spec 139 execution policy.

---

## 21. Enforcement Plan

```yaml
schema: focusa.agent_enforcement_plan.v1
plan_id:
constitution_ref:
controls:
  - claim_ref:
    enforcement_kind: daemon_policy | operation_registry | permission | hook | shell_guard | ci | branch_protection | advisory_only
    target_ref:
    configuration_ref:
    verification_ref:
```

Rules that must never fail cannot remain `advisory_only` without an operator-approved variance.

Examples:

- no local Rust build → Spec 139 execution policy + Pi/shell preflight + process backstop;
- secrets unreadable → target permission deny + Focusa path policy;
- no direct production deploy → operation registry + lease/admission + CI/branch protections;
- formatter after edit → deterministic hook;
- choose architecture based on context → prompt/skill guidance.

---

## 22. Agent Runtime Studio UI

The UI reuses Spec 135I A2UI generated surfaces and typed action bindings.

### 22.1 Project and role

- approved C.R.I.S.T. context/role/spec/task summary;
- agent identity, expertise, responsibilities, non-responsibilities;
- grounding source for every field;
- vertical/domain-pack selection.

### 22.2 Instruction sources

Tree view showing:

- path/source;
- scope;
- authority/trust/freshness;
- applicable targets;
- extracted claims;
- conflicts, duplicates, blind references, and smells.

### 22.3 Conflict workbench

For each conflict:

- exact claims and source excerpts;
- applicability overlap;
- authority comparison;
- operational consequence;
- recommended resolution;
- select/merge/specialize/extract/enforce/reject/edit/defer actions.

### 22.4 Prompt composition

Section tree:

```text
Pi harness foundation
Constitutional Kernel
Project mission
Vertical specialization
Approved role
Authority model
Operating doctrine
Tool protocol
Skill protocol
Workflow
Environment binding
Evidence/completion
Recovery
```

Each section shows source, authority, token budget, mutability class, environment applicability, and redline.

### 22.5 Prompt mode

```text
Preserve Pi default + append
Full Pi replacement
Focusa runtime-compiled
```

### 22.6 Environment variants

Preview exact stable prompt and operating contract for each approved profile, such as:

```text
Mac implementation agent
Mac review agent
remote build agent
integration agent
release agent
browser research agent
```

### 22.7 Tools and skills

For each tool/skill:

```text
active
available_not_exposed
profile_restricted
preflight_required
operator_only
forbidden
missing
unhealthy
```

### 22.8 Boundaries and execution routes

Editable project configuration for:

- source edit venues;
- build/test/integration/release/deploy venues;
- fallback behavior;
- secrets/private/generated files;
- external communications;
- database and production actions.

### 22.9 Output targets

Selectable outputs:

```text
root AGENTS.md
nested AGENTS.md
.pi/SYSTEM.md
.pi/APPEND_SYSTEM.md
Pi runtime compiler profile/extension
CLAUDE.md
.claude/rules
.claude/skills
.claude/settings
GEMINI.md
Copilot instructions
cross-agent skills
enforcement hooks/policies
Focusa bootstrap profile
```

### 22.10 Delivery

```text
copy
export/download bundle
write selected files
create branch
open pull request
apply machine-local profile
rollback prior version
```

Every consequential write follows preview → diff → capability/permission/scope check → confirmation → commit → verification → Receipt.

---

## 23. Prompt and instruction security

### 23.1 Allowed prompt sources

- activated Spec 16 Constitution;
- approved C.R.I.S.T. claims, role, answers, and Spec sections;
- registered domain-pack content;
- verified tool/skill metadata;
- approved operator edits;
- privacy-safe environment/profile projections;
- canonical Focusa authority summaries.

### 23.2 Prohibited direct sources

- raw web pages;
- unapproved uploaded documents;
- arbitrary repository comments;
- issues/PR text treated as instruction;
- terminal output;
- external prompt-like content;
- unresolved contradictions;
- secrets/credentials;
- raw private operator data;
- hidden chain-of-thought;
- stale dynamic presence/time facts.

### 23.3 InstructionInjectionRecord

```yaml
schema: focusa.instruction_injection_record.v1
record_id:
source_ref:
detected_pattern:
trust_class:
affected_candidate_claims: []
disposition: ignore | sanitize | quarantine | operator_review
analysis:
evidence_refs: []
```

### 23.4 Prompt integrity

Compiled prompts include source hashes, compiler version, required sections, forbidden content classes, and a final hash. Runtime delivery verifies the hash before activation.

---

## 24. Versioning, activation, and session pinning

### 24.1 Immutability

Approved Runtime Constitution and PromptVariant versions are immutable. Revisions create new objects.

### 24.2 Session pinning

A running Pi session records:

```text
runtime_constitution_id/revision/hash
prompt_variant_id/hash
assembly_plan_id
compiler_version
environment_profile_ref
tool_inventory_hash
skill_inventory_hash
```

The session continues with its pinned stable version. Material revisions apply only to a new governed session/fork/restart.

### 24.3 Emergency revocation

A severe security or policy incident may revoke a prompt/contract version. Running sessions receive a hard stop/restart requirement rather than silent prompt mutation.

### 24.4 Rollback

Rollback reactivates a prior immutable approved version and produces a Receipt.

---

## 25. Dynamic awareness interlock

The stable prompt contains laws such as:

```text
Before consequential action, obtain fresh Temporal, Presence,
Environment, and Execution Placement guards from Focusa.

Do not assume you are the only active actor.
Do not execute on an unauthorized venue.
Do not duplicate an equivalent active operation.
Failure to observe another actor is not proof of absence.
```

It MUST NOT contain dynamic statements such as current actor names, current build IDs, lease expiry, current time, branch, or resource pressure.

At runtime:

```text
Spec 140 stable Runtime Constitution
+ Spec 139 environment/presence/admission
+ Spec 137 temporal guard
+ operator ask and Workpoint
= current agent behavior
```

---

## 26. Evaluation and champion/challenger governance

### 26.1 Required variants

At minimum, evaluate:

```text
Pi default
Pi default + Focusa append
Focusa full replacement candidate
Focusa runtime-compiled candidate
```

### 26.2 Evaluation dimensions

```text
role fidelity
project understanding
authority adherence
scope safety
tool selection
skill activation
presence awareness
temporal awareness
execution placement
duplicate-work avoidance
multi-agent coordination
evidence behavior
completion truth
prompt-injection resistance
secret handling
context efficiency
prompt-cache stability
compaction recovery
model/provider portability
environment transfer
operator friction
```

### 26.3 Evaluation record

```yaml
schema: focusa.prompt_evaluation.v1
evaluation_id:
variant_ref:
baseline_refs: []
scenario_suite_ref:
environment_refs: []
model_refs: []
metrics: []
failures: []
evidence_refs: []
conclusion:
promotion_recommendation:
```

### 26.4 Spec 138 integration

Prompt identity, version, source manifest, environment, topology, prediction, outcome, calibration, transfer, drift, and negative transfer are preserved. Reflection proposes changes; operator-governed evaluation and approval activate them.

### 26.5 No automatic self-rewrite

The runtime agent cannot edit or activate its own stable system prompt or Runtime Constitution. It may submit evidence-backed proposals.

---

## 27. Drift and impact assessment

Triggers include:

```text
instruction source edited
new nested rule or skill
CI/package script changed
tool/API/CLI deprecated
new machine/profile/daemon
execution route changed
C.R.I.S.T. role/spec amended
domain pack changed
harness version changed
prompt compiler changed
repeated agent failure linked to missing/conflicting instruction
```

```yaml
schema: focusa.agent_contract_impact_assessment.v1
trigger_ref:
affected:
  instruction_claims: []
  runtime_sections: []
  prompt_variants: []
  target_artifacts: []
  skill_bindings: []
  tool_routes: []
  enforcement_controls: []
severity: informational | review | blocker
recommended_actions: []
operator_approval_required:
```

No approved artifact is silently regenerated.

---

## 28. Persistence and events

Required stores:

```text
instruction_sources
instruction_claims
instruction_conflicts
instruction_resolutions
runtime_constitutions
operating_contracts
prompt_assembly_plans
prompt_variants
path_instruction_policies
skill_activation_plans
tool_routing_plans
validation_matrices
enforcement_plans
target_capability_profiles
delivery_manifests
prompt_evaluations
contract_impact_assessments
```

Minimum events:

```text
instruction.source_discovered
instruction.source_changed
instruction.claim_extracted
instruction.conflict_detected
instruction.conflict_resolved
runtime_constitution.drafted
runtime_constitution.reconciled
runtime_constitution.approved
runtime_constitution.activated
runtime_constitution.revoked
prompt.variant_compiled
prompt.variant_evaluated
prompt.variant_approved
artifact.previewed
artifact.delivered
artifact.delivery_verified
artifact.delivery_failed
contract.drift_detected
contract.rollback_activated
```

Consequential transitions require Evidence and Spec 119 Receipts.

---

## 29. Delivery manifest

```yaml
schema: focusa.agent_runtime_delivery_manifest.v1
manifest_id:
constitution_ref:
targets:
  - target_kind:
    target_version:
    artifact_refs: []
    expected_paths: []
    machine_local:
    verification_method:
    enforcement_refs: []
preview_diff_refs: []
approval_ref:
delivery_receipt_refs: []
rollback_ref:
status:
```

Delivery verifies not only that a file exists, but that the target loader recognized the intended revision when the harness exposes verification.

---

## 30. API surface

### Discovery and analysis

```text
POST /v1/agent-runtime/instructions/scan
GET  /v1/agent-runtime/instructions/sources
GET  /v1/agent-runtime/instructions/claims
GET  /v1/agent-runtime/instructions/conflicts
POST /v1/agent-runtime/instructions/reconcile
POST /v1/agent-runtime/instructions/simulate
GET  /v1/agent-runtime/instructions/effective
GET  /v1/agent-runtime/instructions/drift
```

### Runtime Constitution

```text
POST /v1/agent-runtime/constitutions/draft
GET  /v1/agent-runtime/constitutions/:id
POST /v1/agent-runtime/constitutions/:id/preview
POST /v1/agent-runtime/constitutions/:id/approve
POST /v1/agent-runtime/constitutions/:id/activate
POST /v1/agent-runtime/constitutions/:id/revoke
POST /v1/agent-runtime/constitutions/:id/rollback
```

### Compilation

```text
POST /v1/agent-runtime/compile/system-prompt
POST /v1/agent-runtime/compile/agents-md
POST /v1/agent-runtime/compile/skills
POST /v1/agent-runtime/compile/target
GET  /v1/agent-runtime/variants/:id
```

### Evaluation and delivery

```text
POST /v1/agent-runtime/evaluations
GET  /v1/agent-runtime/evaluations/:id
POST /v1/agent-runtime/delivery/preview
POST /v1/agent-runtime/delivery/commit
POST /v1/agent-runtime/delivery/verify
GET  /v1/agent-runtime/delivery/status
```

---

## 31. CLI surface

```text
focusa agent-runtime scan
focusa agent-runtime sources
focusa agent-runtime claims
focusa agent-runtime conflicts
focusa agent-runtime reconcile
focusa agent-runtime simulate --path <path> --profile <profile> --target <target>
focusa agent-runtime effective
focusa agent-runtime drift

focusa agent-runtime constitution draft
focusa agent-runtime constitution show
focusa agent-runtime constitution diff
focusa agent-runtime constitution approve
focusa agent-runtime constitution activate
focusa agent-runtime constitution rollback

focusa agent-runtime prompt compile --target pi --mode append|replace|runtime
focusa agent-runtime prompt preview
focusa agent-runtime prompt evaluate

focusa agent-runtime artifacts compile
focusa agent-runtime artifacts preview
focusa agent-runtime artifacts apply
focusa agent-runtime artifacts verify
focusa agent-runtime doctor
```

All outputs support structured JSON and exact source/authority explanations.

---

## 32. Pi tool surface

Minimum tools:

```text
focusa_agent_runtime_effective
focusa_instruction_sources
focusa_instruction_conflicts
focusa_instruction_explain
focusa_instruction_simulate
focusa_runtime_constitution_preview
focusa_prompt_variant_preview
focusa_prompt_variant_diff
focusa_agent_artifact_preview
focusa_agent_artifact_delivery
focusa_agent_artifact_verify
focusa_agent_runtime_doctor
```

Pi may preview and propose. Approval/activation/delivery authority follows capability, permission, operator, and Receipt requirements.

---

## 33. Migration

### Phase 0 — Inventory and quarantine

- inventory all current instruction/prompt/skill/runbook sources;
- hash and classify them;
- extract claims without changing behavior;
- identify contradictions and current effective harness loading;
- label legacy and stale sources.

### Phase 1 — Canonical contracts

- implement InstructionSource, InstructionClaim, conflict graph, Runtime Constitution, Operating Contract, Prompt Assembly Plan, Skill/Tool/Validation/Enforcement plans.

### Phase 2 — C.R.I.S.T. integration

- add post-task-materialization Runtime Constitution stages;
- compile approved role/spec/context references;
- add grounding and operator review.

### Phase 3 — AGENTS/rules/skills

- root and nested AGENTS compiler;
- target capability profiles;
- Claude/Gemini/Copilot/Pi skill and rule projections;
- delivery preview/verify.

### Phase 4 — Pi prompt compiler

- append mode;
- full replacement mode;
- runtime-compiled mode;
- session pinning and cache safety;
- safe fallback.

### Phase 5 — Enforcement

- compile operation registry, permissions, hooks, Spec 139 policies, CI/branch controls;
- prove hard rules do not rely solely on prose.

### Phase 6 — Runtime Studio and evaluation

- A2UI UI;
- champion/challenger benchmark suite;
- Spec 138 integration;
- drift, rollback, and multi-machine profile deployment.

---

## 34. Required tests

### Discovery and parsing

1. Discover root and nested AGENTS.md.
2. Discover Pi SYSTEM/APPEND_SYSTEM and skills.
3. Discover Claude/Gemini/Copilot sources.
4. Distinguish project, local, user, and managed sources.
5. Raw code comments do not become authoritative claims.
6. Prompt-like untrusted content is quarantined.
7. Atomic claims preserve source and applicability.

### Authority and conflicts

8. Direct contradiction blocks compilation.
9. Conditional build-route contradiction is resolved by environment-specific validation matrix.
10. Nested rule cannot weaken root hard prohibition.
11. Operator-approved scoped exception works without universal weakening.
12. Stale command/tool/reference is detected.
13. Blind reference is rejected or annotated with why/when.
14. Duplicate semantic claims are consolidated without losing provenance.
15. Role text cannot grant permission.

### Prompt compilation

16. Pi append preserves default prompt and adds approved sections.
17. Full replacement contains required sections and only actual tools.
18. Runtime-compiled prompt is stable across turns.
19. Dynamic presence/time changes do not change stable prompt hash.
20. Environment-profile change requires rebind/new session.
21. Focusa outage uses safe fallback, not stale full authority.
22. Prompt source hashes and compiler version verify.
23. Secret/raw private data does not appear.
24. Unapproved C.R.I.S.T. claims do not appear.

### AGENTS and cross-harness

25. Root AGENTS stays within budget.
26. Nested AGENTS contains deltas only.
27. Claude target imports common AGENTS and emits path rules.
28. Gemini target preserves hierarchical/JIT semantics.
29. Copilot target emits valid repository/path instructions.
30. Unsupported target feature is reported, not faked.
31. Equivalent canonical intent is preserved across targets.

### Skills/tools/enforcement

32. Skill metadata loads without full-body bloat.
33. Side-effecting skill is operator-only.
34. Missing tool dependency blocks skill delivery.
35. Tool route uses structured result schema.
36. Hard no-local-build claim maps to Spec 139 policy/hook, not advisory-only.
37. Permission/hook output validates against target schema.
38. Enforcement verifier proves target installation.

### C.R.I.S.T. and UI

39. Runtime Constitution cannot draft before required approved inputs.
40. Operator can edit/regenerate one section without silent global rewrite.
41. Conflict workbench preserves source and recommendation.
42. Prompt diff shows default/append/replacement variants.
43. Per-environment previews differ only where policy requires.
44. Copy/export/write/branch/PR delivery share one approved manifest.
45. Reopen/resume preserves progress.

### Evaluation/versioning

46. Default, append, replacement, and runtime variants run same scenario suite.
47. Prompt-cache stability is measured.
48. Compaction reloads active stable contract and dynamic awareness separately.
49. Session remains pinned to original approved version.
50. New approval affects only new sessions.
51. Revocation forces governed stop/restart.
52. Rollback reactivates prior immutable version.
53. Spec 138 records prompt identity, environment, outcome, transfer, and negative transfer.

### Incident prevention

54. Contradictory `do not build locally` and `run all builds` cannot compile unresolved.
55. Mac implementation prompt states remote validation route but does not embed volatile build status.
56. Spec 139 enforcement blocks local Cargo process.
57. Remote build agent receives the appropriate role/skill/tool/validation profile.
58. Equivalent active run produces subscription behavior.

### Security/privacy

59. Malicious imported document cannot inject system instructions.
60. Machine-local secret paths are redacted from committed artifacts.
61. Managed policy cannot be weakened by project output.
62. Delivery cannot overwrite target files without preview/approval.
63. Target-loader verification failure prevents completion claim.
64. Cross-project contract bleed is impossible.

---

## 35. Acceptance criteria

Spec 140 is accepted only when:

1. C.R.I.S.T. can produce a grounded draft Runtime Constitution after approved Project Genesis inputs.
2. Existing instruction sources are comprehensively discovered and classified.
3. Prose is converted into typed atomic claims with provenance and applicability.
4. Authority/preference and target capability graphs are explicit.
5. Contradictions and instruction smells are detected precisely.
6. Operator reconciliation is reviewable, editable, and durable.
7. Spec 16/16B ownership remains intact.
8. Role, capability, permission, and authority remain separate.
9. Pi append, replacement, and runtime-compiled modes operate.
10. Stable/session/dynamic prompt layers remain separated.
11. Dynamic presence/time facts never destabilize the stable prompt prefix.
12. Root and nested AGENTS.md compile from the same canonical contract.
13. Skills use progressive disclosure and side-effect governance.
14. Tool/API/CLI routing reflects actual capabilities.
15. Vague quality-gate prose is replaced by a typed validation matrix.
16. Hard rules compile to deterministic enforcement where supported.
17. Claude, Gemini, Copilot, Pi, and generic target outputs truthfully preserve capability limits.
18. Agent Runtime Studio supports source review, conflict resolution, prompt composition, profile variants, delivery, and rollback.
19. Every artifact is previewed, diffed, approved, delivered, and verified.
20. Prompt variants are benchmarked against baselines before activation.
21. Sessions pin immutable approved versions.
22. Drift creates impact assessments rather than silent regeneration.
23. Secrets, raw private data, and instruction injection are blocked.
24. Evidence and Receipts prove approval, delivery, verification, activation, and rollback.
25. The local dual-build instruction conflict is deterministically detected and cannot recur through an approved compiled contract.
26. No mandatory behavior exists only in an unverified prompt or Markdown file.

---

## 36. Machine-readable requirement ledger

Every normative statement MUST map to `docs/contracts/spec140-complete-feature-ledger.v1.yaml` before implementation closure.

Required row shape:

```yaml
requirement_id:
spec_section:
requirement_text:
requirement_class: must | shall | should | may
applicability: required | conditional | optional | not_applicable
applicability_condition_ref:
primitive_owner:
implementation_slice:
blocking_dependencies: []
core_types: []
reducer_events: []
persistence: []
api_operations: []
cli_commands: []
pi_tools: []
ui_surfaces: []
target_compilers: []
enforcement_outputs: []
operation_registry_changes: []
generated_contracts: []
migrations: []
positive_tests: []
negative_tests: []
restart_recovery_tests: []
security_tests: []
performance_tests: []
evaluation_scenarios: []
evidence_refs: []
receipt_refs: []
status: not_started | active | blocked | implemented_unverified | verified | variance_approved | not_applicable_verified
```

A normative clause without a ledger mapping fails completeness.

---

## 37. Canonical summary

```text
C.R.I.S.T.
  determines the approved project, role, context, spec, and tasks.

Spec 16
  supplies the declarative Constitutional Kernel.

Spec 140
  reconciles instruction sources and compiles the stable agent identity,
  system prompt, AGENTS hierarchy, skills, tool routing, validation matrix,
  enforcement plan, and target-specific artifacts.

Specs 137 and 139
  supply fresh time, environment, presence, topology, placement, and admission.

Workpoint
  supplies immediate action authority.
```

The result is not a generic coding assistant with a longer prompt. It is a governed, project-born, vertical-specialized agent whose stable operating identity is approved through C.R.I.S.T., whose instructions are internally consistent, whose target artifacts are verified, and whose live actions remain constrained by Focusa’s current distributed reality.