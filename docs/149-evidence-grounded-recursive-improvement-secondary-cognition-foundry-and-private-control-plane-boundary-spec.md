# Spec 149 — Evidence-Grounded Recursive Improvement, Secondary Cognition Foundry, Champion/Challenger Optimization, and Private Control-Plane Boundary

**Status:** Normative architecture specification; implementation and proof pending; private Focusa Foundry deployment not implied; no consumer packaging authority  
**Owner:** Focusa core governance and development operations  
**Created:** 2026-07-30  
**Canonical label:** **Spec 149 — Focusa Recursive Improvement Foundry**  
**Source baseline reviewed:** `2f25e1b967ccb009e7e70d2a1202d1b99f7980c7`  
**Specializes:** Spec 78 bounded secondary cognition and persistent autonomy  
**Depends on:** Specs 16, 16B, 51, 54A, 54B, 56, 57, 61, 67–79, 88, 96, 100, 101, 104, 107, 113, 114, 119, 120, 125, 130, 130A, 131, 133, 136, 137, 137A, 138, 138A, 140, 140A, 141, 142, 143, 144, 145–148  
**Primary public surfaces:** Focusa core contracts, Workpoint, Trajectory, Evidence/ECS, Receipts, Secondary Cognition, Prediction/Metacognition, benchmarks, Context Authority, Silent Sessions, release governance, provider/harness capability contracts, and public/private boundary documentation  
**Private deployment surfaces:** deployment-defined external service, private repository, private schedules, private credentials, private research corpus, private candidate ledger, private holdout evaluations, and private operator policy

---

## 0. One-line definition

The **Focusa Recursive Improvement Foundry** is a governed, evidence-producing, continuously resumable improvement program that uses Focusa Secondary Cognition to discover, test, compare, retain, reject, and formulate improvements without granting the recursive process sovereign code, product, issue, or release authority.

---

## 1. Executive requirement

Focusa MUST support a recursive improvement architecture capable of continuously examining a software project’s:

- codebase;
- architecture;
- database and persistence model;
- runtime and server behavior;
- harness and provider integrations;
- context-management and compaction behavior;
- specifications and implementation trajectories;
- tests, benchmarks, evidence, incidents, and releases;
- duplicate, obsolete, inefficient, or unreachable code;
- resource consumption and performance;
- security, privacy, recovery, and correctness boundaries.

The architecture MUST transform observations into bounded, evidence-linked **Improvement Candidates** and, when separately authorized, high-quality GitHub issues or other work-provider items.

It MUST NOT operate as unconstrained self-modification.

The central law is:

> **Discovery may be continuous and autonomous. Canonical mutation, issue publication, policy promotion, release admission, and production deployment remain separately governed.**

A recursive process may become increasingly effective at finding and validating opportunities. It may not silently make itself more authoritative.

---

## 2. Relationship to Spec 78 Secondary Cognition

### 2.1 Spec 78 remains the primitive owner

Spec 78 owns the foundational contract for:

- bounded secondary cognition;
- persistent autonomous continuation;
- operator-first interruption;
- ontology and scope governance;
- immutable evaluation surfaces;
- cheap bounded experiments;
- explicit keep/discard advancement;
- exhaustive result logging;
- promotion, rejection, projection retention, and failed-attempt archival;
- continuation and invalidation conditions;
- advisory worker output;
- trace, checkpoint, recovery, retention, and decay.

Spec 149 MUST NOT create a second recursive-cognition substrate.

Spec 149 specializes Spec 78 for **whole-project and whole-product improvement**.

```text
Spec 78 Secondary Cognition
  owns how subordinate cognition observes, proposes, verifies,
  projects, reflects, predicts, continues, stops, and records outcomes.

Spec 149 Recursive Improvement Foundry
  owns how those primitives are composed into a continuous software,
  architecture, compression, performance, and release-improvement program.
```

### 2.2 Autoresearch inheritance

Spec 78 already adopts selected principles from Karpathy’s `autoresearch`:

- immutable evaluation harness;
- a constrained mutable experiment surface;
- fixed or bounded experiment budgets;
- persistent iterations;
- explicit keep/discard/revert decisions;
- complete result logging;
- preservation of crashes and failures as evidence.

Spec 149 carries those principles forward and generalizes them from one model-training file to a governed software project.

External conceptual authority:

- `https://github.com/karpathy/autoresearch`
- `https://github.com/karpathy/autoresearch/blob/master/program.md`

The upstream project is an inspiration, not Focusa authority. Focusa’s own specs, reducers, Workpoints, Evidence, Receipts, Context Authority, and release rules remain normative.

### 2.3 Required Secondary Cognition roles

The Foundry MUST express recursive work through typed Spec 78 roles:

```text
Scout / Observer
  gathers bounded facts and source references.

Extractor / Proposer
  creates a non-canonical Improvement Candidate.

Critic / Adversarial Verifier
  tries to falsify the candidate and its claimed benefit.

Projection Builder
  creates bounded views without mutating canonical truth.

Predictor
  forecasts expected benefit, cost, risk, and transferability.

Experimenter
  runs one bounded challenger experiment against an immutable harness.

Outcome Evaluator
  settles measurements against frozen success criteria.

Retention Advisor
  recommends active, decayed, archived, or superseded posture.
```

No role may combine proposal, verification, promotion, and release authority into one opaque decision.

---

## 3. Research and production basis

This specification is grounded in current Focusa architecture and external primary sources.

### 3.1 GPT-5.6 Sol and reasoning continuity

OpenAI reported that retaining reasoning state and using Responses API compaction materially improved GPT-5.6 Sol performance on ARC-AGI-3 while reducing output-token usage. The reported harness failure modes included discarding reasoning and rolling truncation.

References:

- `https://openai.com/index/how-two-settings-tripled-our-arc-agi-3-scores/`
- `https://developers.openai.com/api/docs/guides/compaction`

The resulting Focusa rule is:

```text
provider cognition remains provider-owned;
Focusa canonical governance remains Focusa-owned;
prompt-facing projections remain bounded derived views.
```

Opaque provider reasoning or compaction objects MUST be persisted and replayed according to the provider contract. Focusa MUST NOT parse, summarize, edit, or replace opaque provider cognition.

### 3.2 Context retrieval and compression

Relevant primary research includes:

- LongLLMLingua — query-aware long-context compression: `https://arxiv.org/abs/2310.06839`
- LongMemEval — structured long-term memory and retrieval evaluation: `https://arxiv.org/abs/2410.10813`
- Addressable Recall Compaction — exact addressable archival recall: `https://arxiv.org/abs/2607.25066`
- prompt-compression reasoning-stability cautions: `https://arxiv.org/abs/2605.17932`

Focusa may borrow principles from these systems, but compression remains inside deterministic authority, evidence, and recovery envelopes.

### 3.3 Confidence-scheduled work

DSpark uses confidence-scheduled verification to allocate expensive work where prefix survival is likely to justify it:

- `https://arxiv.org/abs/2607.05147`

Unless Focusa controls model serving, DSpark’s decoding kernel is not directly portable. Its scheduling principle is portable:

```text
spend deeper verification and experimentation where expected candidate value,
uncertainty, survival probability, and evidence quality justify the cost.
```

### 3.4 Safe adaptive policy selection

Relevant safe-policy and conservative exploration research includes:

- conservative contextual bandits;
- high-confidence off-policy evaluation;
- bounded policy routing over validated alternatives.

Reference:

- `https://arxiv.org/abs/2002.00467`

Focusa MUST keep a verified production champion available and MUST NOT use unconstrained online exploration for canonical production policy.

---

## 4. Scope

### 4.1 In scope

Spec 149 owns:

1. The `RecursiveImprovementProgram` umbrella contract.
2. Bounded `ImprovementCycle` identity and lifecycle.
3. `ImprovementCandidate` identity, state, evidence, and disposition.
4. Composition of Spec 78 Secondary Cognition roles for project improvement.
5. Continuous research and repository discovery under fixed authority.
6. Release-aware and concurrent-agent collision handling.
7. Candidate novelty, duplication, supersession, and in-flight-work classification.
8. Champion/challenger experiments and outcome settlement.
9. Compression investigation across provider, semantic, prompt, transport, and storage planes.
10. Code-reachability and replacement-before-removal governance.
11. GitHub issue formulation and publication gates.
12. Private Foundry execution-profile boundaries.
13. Consumer-package exclusion requirements.
14. Resource budgets, checkpointing, restart recovery, and observability.
15. Cross-model and cross-harness evaluation.
16. Learning transfer into later cycles through Specs 138 and 148.

### 4.2 Out of scope

Spec 149 does not own:

- Workpoint or Trajectory authority;
- canonical project identity;
- operator approval;
- release scope, tagging, deployment, or rollback authority;
- direct model-weight modification;
- hidden chain-of-thought capture;
- provider-native reasoning formats;
- the generic Silent Session implementation;
- the generic Secondary Cognition implementation;
- domain-specific market, medical, legal, or security beliefs;
- an always-installed consumer background service;
- automatic activation of a private Focusa self-improvement worker;
- unrestricted code mutation;
- issue-count maximization.

---

## 5. Foundational laws

1. **Spec 78 owns the loop substrate.** Spec 149 composes it for project improvement.
2. **The loop is not sovereign.** Operator, safety, scope, and canonical authority outrank loop momentum.
3. **Discovery and mutation are separate authorities.** Discovering an opportunity does not authorize changing code.
4. **Proposal and verification are separate roles.** A proposer cannot certify its own claim merely by restating it.
5. **Evaluation is immutable within an experiment.** A challenger cannot rewrite its own success criteria.
6. **Production behavior is the champion.** A challenger begins non-canonical.
7. **One experiment changes one bounded hypothesis surface.** Broad uncontrolled bundles are not comparable.
8. **Failed experiments remain evidence.** Reversion removes active behavior, not historical truth.
9. **Replacement precedes removal.** No planned or implemented capability may be removed without a stronger replacement, migration, rollback, and proof.
10. **Unused is not equivalent to obsolete.** Reachability requires classification and intent evidence.
11. **Everything is scoped.** Candidates, research, tests, handles, Workpoints, predictions, and lessons remain inside exact project/workstream authority.
12. **Provider cognition is opaque when the provider says it is opaque.** Focusa never invents introspection into encrypted or signed provider state.
13. **Token compression, semantic compression, and byte compression are different optimization planes.** Gains in one cannot be assumed in another.
14. **Binary encodings do not belong in hot model prompts.** Decode before model-facing projection.
15. **Issue publication is consequential mutation.** It requires novelty, evidence, scope, and publication authority.
16. **A cycle with no new issue may be successful.** Proving that existing work already covers a finding is valuable closure.
17. **Continuous does not mean unbounded.** Execution occurs through finite, checkpointed cycles.
18. **Private operation does not become hidden product behavior.** The public boundary must remain inspectable.
19. **Consumer installs do not receive Focusa’s private self-improvement worker.** Generic primitives may ship; the privileged deployment does not.
20. **Learning is evaluated.** Repetition is not evidence of improvement.

---

## 6. Public and private architecture boundary

### 6.1 Public Focusa capability

The public product may provide generic primitives required by any governed improvement program:

- Workpoints;
- Trajectory;
- Evidence and ECS;
- Receipts;
- Secondary Cognition;
- Prediction and Metacognition;
- Context Authority;
- Silent Sessions;
- benchmark and eval ledgers;
- capability discovery;
- provider/harness adapters;
- release governance;
- content-addressed storage;
- policy evaluation and rollback primitives.

### 6.2 Private Focusa development deployment

Focusa’s own continuously running Foundry MAY be deployed as an operator-private external service.

Its private implementation includes:

- service code;
- operating prompt and research policy;
- schedules and cadence;
- credentials and GitHub scopes;
- private repository access;
- private candidate ledger;
- private research corpus;
- private benchmark holdouts;
- private cost, threshold, and publication policy;
- server topology and process supervision;
- raw unsanitized evidence;
- private operator strategy.

These private details MUST NOT be required in the public specification and MUST NOT be committed to public product surfaces.

### 6.3 Consumer package exclusion

The following MUST NOT be included in ordinary consumer release artifacts unless a future explicit product specification and operator opt-in supersedes this rule:

```text
focusa-self-improvement service or timer
Focusa-repository-specific recursive worker
private Foundry prompts or policies
private credentials or GitHub publication authority
private research corpus
private candidate database
private release observer configuration
private holdout evals
private server topology
```

Consumer installers MUST NOT silently start a repository-improvement worker.

### 6.4 Deployment-profile reference

Public records may carry an opaque reference:

```yaml
private_execution_profile_ref: private://deployment-defined
```

The reference is not authorization, a path, a secret, or a requirement to publish private configuration.

---

## 7. Canonical object model

### 7.1 `RecursiveImprovementProgramV1`

```yaml
schema: focusa.recursive_improvement_program.v1
program_id:
project_ref:
project_root_key:
trajectory_ref:
hlt_ref:
program_workpoint_ref:
program_spec_ref:
baseline_release_ref:
baseline_commit_sha:
secondary_cognition_policy_ref:
research_policy_ref:
benchmark_policy_ref:
publication_policy_ref:
private_execution_profile_ref:
discovery_domains: []
allowed_outputs: []
forbidden_actions: []
resource_policy_ref:
retention_policy_ref:
status: draft | approved | observing | active | paused | blocked | settling | archived
created_by:
approved_by:
created_at:
revision:
```

The program is an umbrella governance record. It does not contain provider secrets, GitHub tokens, server paths, or raw prompts.

### 7.2 `ImprovementCycleV1`

```yaml
schema: focusa.improvement_cycle.v1
cycle_id:
program_id:
source_commit_sha:
source_release_ref:
release_lock_ref:
workpoint_ref:
trajectory_ref:
secondary_session_refs: []
started_at:
settled_at:
state:
inspected_domains: []
research_snapshot_ref:
repository_snapshot_ref:
candidate_refs: []
eval_refs: []
evidence_refs: []
receipts: []
resource_usage:
continuation_decision:
stop_reason:
next_safe_action:
```

### 7.3 `ImprovementCandidateV1`

```yaml
schema: focusa.improvement_candidate.v1
candidate_id:
program_id:
cycle_id:
source_commit_sha:
root_cause:
failure_mode:
current_behavior:
proposed_replacement:
affected_primitives: []
affected_surfaces: []
affected_specs: []
planned_functionality_refs: []
preserved_functionality: []
research_claim_refs: []
evidence_refs: []
reproduction_refs: []
benchmark_refs: []
related_issue_refs: []
related_bead_refs: []
related_pr_refs: []
release_collision_status:
provider_scope:
harness_scope:
expected_benefits: {}
expected_costs: {}
confidence:
novelty_status:
verification_status:
publication_status:
retention_status:
created_at:
settled_at:
```

### 7.4 Candidate states

```text
observed
researching
reproducing
challenged
validated
benchmarking
duplicate
covered_by_spec
covered_by_release
in_flight_elsewhere
superseded
not_reproducible
eligible_for_issue
withheld
published
implemented_elsewhere
settled
archived
```

A candidate is not a GitHub issue. It becomes issue-eligible only after the publication gate passes.

---

## 8. Architecture

```text
Operator / Project Governance / Release Authority
                         │
                         ▼
             RecursiveImprovementProgram
                         │
                         ▼
                 ImprovementCycle
       ┌─────────────────┼──────────────────┐
       ▼                 ▼                  ▼
 Repository Scout   Research Scout   Runtime/Benchmark Scout
       │                 │                  │
       └──────────────┬──┴───────────────┬──┘
                      ▼                  ▼
          Candidate Extractor      Evidence Store
                      │                  │
                      ▼                  │
           Adversarial Verifier ◄────────┘
                      │
                      ▼
            Collision/Novelty Resolver
                      │
             ┌────────┴─────────┐
             ▼                  ▼
      Champion/Challenger   Existing Work Mapping
             │                  │
             ▼                  ▼
        Outcome Settlement / Candidate Disposition
                         │
                 ┌───────┴────────┐
                 ▼                ▼
          Issue-eligible       Withhold/archive/
          proposal             attach to existing work
```

All model-backed boxes are Secondary Cognition roles. None independently owns canonical mutation.

---

## 9. Cycle lifecycle

### 9.1 States

```text
created
source_resolving
scope_verifying
observing
researching
discovering
candidate_forming
reproducing
adversarial_verifying
collision_resolving
benchmarking
settling
publication_pending
settled
paused
blocked
aborted
```

### 9.2 Required sequence

```text
1. resolve remote repository and release reality;
2. verify project/workstream authority;
3. resume the program Workpoint and Trajectory;
4. freeze source SHA and research snapshot;
5. discover through bounded Secondary Cognition workers;
6. externalize raw output to Evidence/ECS;
7. form typed candidates;
8. adversarially test candidates;
9. search issues, Beads, specs, PRs, commits, and release work;
10. run bounded differential experiments when justified;
11. settle candidate outcomes;
12. publish only separately authorized eligible issues;
13. evaluate predictions and learning;
14. checkpoint and decide whether another cycle is justified.
```

### 9.3 Continuation conditions

A subsequent cycle may begin only when:

- no newer operator direction supersedes the program;
- scope remains verified;
- the Workpoint permits continuation;
- resource budgets permit another cycle;
- the previous cycle settled or explicitly checkpointed;
- a new source change, research change, incident, benchmark signal, or justified unexplored domain exists;
- no stop condition is active.

### 9.4 Stop and pause conditions

The Foundry MUST pause or stop on:

- operator steering;
- scope conflict;
- release/worktree ownership conflict;
- repeated candidate set without new evidence;
- repeated experiment without new hypothesis;
- compromised or mutable eval harness;
- resource-budget exhaustion;
- degraded provider capability;
- evidence-store or receipt failure;
- issue-publication ambiguity;
- private/public boundary uncertainty;
- security or secret-exposure risk.

---

## 10. Autoresearch-style experiment contract

### 10.1 Immutable harness

Each experiment MUST freeze:

- source baseline SHA;
- workload and fixtures;
- evaluation logic;
- metric definitions;
- resource and time budget;
- stop conditions;
- promotion thresholds;
- security and authority gates.

The experimenter MUST NOT change those surfaces while optimizing the challenger.

### 10.2 Bounded mutable surface

Every experiment declares the exact mutable surface:

```yaml
mutable_files: []
mutable_symbols: []
mutable_config_fields: []
forbidden_files: []
forbidden_behaviors: []
```

One experiment SHOULD isolate one causal hypothesis. Broad refactors require a staged experiment graph.

### 10.3 Fixed or normalized budget

Experiments MUST use a comparable budget, such as:

- fixed wall-clock duration;
- fixed task suite;
- fixed request count;
- fixed token budget;
- fixed event count;
- fixed dataset snapshot;
- fixed server resource class.

Cross-host results MUST identify hardware and workload differences and MUST NOT claim direct comparability without normalization.

### 10.4 Result disposition

Each experiment ends in exactly one primary disposition:

```text
keep_challenger
retain_for_more_evidence
reject_challenger
revert_challenger
archive_crash
blocked_by_harness
invalid_experiment
```

A crash is logged and preserved. It is not silently deleted.

### 10.5 Development-mode mutation boundary

The default Foundry mode is issue-only and read-mostly.

A future operator-approved experiment mode MAY mutate a disposable branch or isolated worktree. It MUST NOT mutate the canonical production branch, running production installation, release tag, or sole recovery copy.

---

## 11. Release and concurrent-agent awareness

### 11.1 Release state is an input, not a blanket prohibition

A locked or active release does not inherently prohibit creating this specification or running unrelated observation. It does affect candidate collision, experiment ownership, and issue novelty.

Required states:

```text
no_active_release_conflict
active_release_observe_safe
active_release_file_overlap
active_release_candidate_overlap
release_settlement_pending
post_release_revalidation_required
```

### 11.2 Exact-SHA revalidation

A candidate discovered against one SHA MUST be revalidated when:

- its affected files changed;
- a related issue closed;
- a related PR merged;
- the active release claims to settle the same root cause;
- the candidate is about to be published after meaningful source movement.

### 11.3 Multiple agents

The Foundry MUST query or infer current ownership through Workpoints, Beads, issues, PRs, worktrees, and release records.

It MUST NOT:

- edit an agent-owned dirty worktree;
- file a duplicate issue because an in-flight change is not yet on `main`;
- treat unpushed local work as nonexistent when Focusa has an authoritative active Workpoint for it;
- claim a release fixed a problem without exact evidence;
- assume every open issue remains unresolved merely because it is open.

---

## 12. Discovery domains

A complete program MAY activate bounded workers for:

1. Rust architecture and reducer ownership.
2. API routes and daemon hot paths.
3. SQLite, persistence, snapshots, WAL, replay, and migration.
4. Pi extension lifecycle, context, and compaction.
5. Codex and other harness adapters.
6. provider reasoning, cache, continuation, and compaction contracts.
7. Evidence/ECS storage and rehydration.
8. Workpoint, Trajectory, Context Authority, and scope isolation.
9. Silent Sessions, process supervision, and resource policy.
10. prompt assembly and instruction diet.
11. tool contracts, generated projections, and parity.
12. code duplication, unreachable code, lint debt, and dependency debt.
13. compression, serialization, transport, and storage encoding.
14. tests, benchmarks, eval leakage, and holdout integrity.
15. install, update, rollback, and release lifecycle.
16. security, privacy, prompt injection, secrets, and cross-project leakage.
17. specifications, implementation drift, and supersession conflicts.
18. user-facing performance, latency, clarity, and operator burden.

Each worker returns bounded results with Evidence handles. Raw logs do not remain in the principal model’s hot prompt.

---

## 13. Code reachability and cleanup governance

### 13.1 Required classification

Apparently unused or duplicated code MUST be classified as:

```text
required_reachable
planned_spec_linked
public_api_compatibility
generated_surface
test_or_fixture
duplicate_with_canonical_replacement
obsolete_with_migration
unknown_requires_investigation
```

### 13.2 Planned-functionality protection

`planned_spec_linked` requires an exact reference to one or more of:

- active specification clause;
- approved issue or Bead;
- Trajectory/Waypoint;
- release requirement;
- compatibility commitment;
- operator-approved design record.

Planned code may be compressed, consolidated, relocated, or replaced when the replacement preserves the requirement.

### 13.3 Replacement-before-removal law

A removal proposal is invalid unless it contains:

- the capability or requirement being preserved;
- a stronger or more coherent replacement;
- behavior and API compatibility analysis;
- data migration when relevant;
- rollback;
- proof of semantic equivalence or improvement;
- affected-spec amendments;
- negative tests proving no planned functionality disappeared.

### 13.4 Lint debt

Global `allow` settings for dead or unused code SHOULD be retired incrementally, crate by crate or module by module, after classification. A lint cleanup MUST NOT become an unreviewed capability deletion campaign.

---

## 14. Compression and performance program

### 14.1 Separate optimization planes

The Foundry MUST distinguish:

```text
Provider cognition plane
  reasoning state, thought signatures, opaque compaction items,
  provider-native continuation.

Canonical semantic plane
  Workpoint, Trajectory, scope, decisions, constraints,
  blockers, Evidence, Receipts, and state revisions.

Prompt projection plane
  token-budgeted model-visible context.

Transport/storage plane
  databases, event logs, sidecars, snapshots, network envelopes,
  binary codecs, and compression.
```

An optimization in one plane MUST NOT be reported as a gain in another without measurement.

### 14.2 Prompt compression

Prompt optimization SHOULD prioritize:

1. current operator ask;
2. verified scope;
3. Workpoint immediate action;
4. HLT/Trajectory posture;
5. exact blocker;
6. active constraints;
7. Evidence and Receipt refs;
8. exact next tool;
9. rehydrate handles.

The Foundry SHOULD detect:

- duplicate packet rendering;
- expanded JSON plus repeated prose;
- repeated tool tutorials;
- stable-prefix churn;
- raw tool-output leakage;
- recursive summary-of-summary growth;
- missing exact rehydration paths;
- compression that preserves semantics but breaks reasoning.

### 14.3 Byte and bit-level compression

The Foundry MAY benchmark:

- unsigned varints or VLQ;
- ZigZag signed integers;
- timestamp and sequence deltas;
- dictionary-coded schema and event IDs;
- sorted/delta-coded reference sets;
- string interning;
- CBOR;
- MessagePack;
- Protobuf-style tagged envelopes;
- Zstandard and trained dictionaries;
- immutable compressed event segments;
- domain-sharded snapshots;
- Merkle/content-addressed manifests.

No codec is preferred by specification.

### 14.4 Binary prompt prohibition

Binary, base64, or compressed payloads MUST NOT be inserted directly into hot model context as a token-saving strategy. Model-facing projections are decoded and rendered through a measured target profile.

### 14.5 Structural waste before codec optimization

The required order is:

```text
remove duplicate state and duplicate writes
→ externalize bulky content
→ shard state and isolate dirty revisions
→ remove duplicate prompt projections
→ preserve provider cache/reasoning continuity
→ benchmark codecs and bit-level encodings
```

Saving integer bytes does not compensate for cloning or rewriting an unnecessarily large aggregate state.

---

## 15. Persistence challenger program

The Foundry SHOULD measure:

- state clone time;
- serialized bytes;
- serialization duration;
- allocation peak;
- writes per event;
- event bytes versus snapshot bytes;
- unchanged-domain rewrite ratio;
- WAL growth;
- checkpoint latency;
- hot-route latency under sustained writes;
- restart and replay latency;
- queue depth and coalescing;
- CPU and RSS.

A candidate challenger may use:

```text
immutable or Arc-backed domain shards
+ dirty-domain revision tracking
+ content-addressed shard snapshots
+ compact root manifest
+ occasional full compatibility checkpoint
```

It MUST prove deterministic replay, accepted-mutation durability, event-chain integrity, migration, rollback, and lower measured cost.

---

## 16. Provider and harness capability contracts

### 16.1 Model names grant no capability

Provider-native behavior is legal only through versioned capability evidence.

Relevant capabilities include:

```text
reasoning_state_round_trip
thought_signature_round_trip
opaque_compaction_round_trip
provider_compaction_block_round_trip
previous_response_continuation
previous_interaction_continuation
full_output_replay
prompt_cache
cache_usage_accounting
structured_tactical_summary
selective_artifact_rehydration
session_resume_survives_process_restart
```

### 16.2 Required scope classification

Each candidate states whether it is:

```text
universal_core
provider_neutral
provider_specific
model_family_specific
model_specific
harness_specific
adapter_specific
operating_system_specific
hardware_specific
```

### 16.3 GPT-5.6 Sol priority without architectural capture

The private Focusa Foundry MAY prioritize GPT-5.6 Sol, Codex, and Pi because they are high-value development paths.

Public Focusa architecture MUST remain provider- and harness-neutral. GPT-5.6 Sol optimizations MUST be expressed through capability contracts and measured fallbacks, not hard-coded assumptions that weaken other models.

---

## 17. Research ingestion and claim integrity

### 17.1 Research claim record

```yaml
schema: focusa.research_claim.v1
claim_id:
source_class: official_provider | primary_paper | production_oss | secondary
source_ref:
published_at:
retrieved_at:
claim:
tested_environment:
reported_effect:
limitations: []
focusa_mapping: []
directly_applicable:
required_experiment:
confidence:
evidence_refs: []
```

### 17.2 Source hierarchy

Prefer:

1. official provider contracts;
2. primary papers;
3. production source and reproducible implementation;
4. high-quality secondary synthesis;
5. informal claims only as investigation leads.

### 17.3 No citation-to-canonical shortcut

A published result is not proof that the same change improves Focusa. It creates a falsifiable candidate and experiment requirement.

### 17.4 Research freshness

The Foundry MUST store retrieval date and version or commit when possible. Provider and harness contracts are temporally unstable and require refresh before consequential implementation.

---

## 18. Candidate scheduling

The Foundry MAY use a confidence-scheduled priority model:

```text
priority increases with:
  expected verified benefit
  evidence quality
  uncertainty reduction
  cross-model breadth
  recurrence frequency
  severity
  strategic fit

priority decreases with:
  validation cost
  release collision risk
  migration risk
  low reproducibility
  existing-work coverage
  weak measurement surfaces
```

This score is advisory. Safety, privacy, data-loss, scope isolation, and authority failures may outrank economic scheduling.

---

## 19. Champion/challenger governance

### 19.1 Champion

The champion is the current verified production behavior or current accepted policy for the exact capability segment.

### 19.2 Challenger lifecycle

```text
hypothesis
→ static validation
→ replay
→ differential benchmark
→ fault injection
→ shadow evaluation
→ bounded canary proposal
→ operator/release admission
→ outcome settlement
→ promote, retain, quarantine, reject, or rollback
```

### 19.3 No in-place self-mutation

A canonical policy or instruction set MUST NOT modify itself in place based solely on its own output.

Changes create a new immutable candidate revision with provenance and rollback.

### 19.4 Pareto evaluation

Candidates SHOULD expose a vector rather than one opaque score:

```text
task success
authority fidelity
reasoning continuity
token use
cache cost
latency
CPU
RSS
storage
write amplification
recovery
cross-harness coverage
operator burden
implementation risk
migration risk
proof strength
```

A token reduction with lower task success or weaker recovery is not automatically an improvement.

---

## 20. Immutable eval and benchmark governance

### 20.1 Frozen within a run

The challenger cannot change:

- golden tasks;
- holdout membership;
- scoring formulas;
- pass/fail conditions;
- evidence requirements;
- resource budget;
- comparison baseline;
- rollback trigger.

### 20.2 Eval-policy proposals

Secondary Cognition may propose a future eval-policy revision through Spec 120/138 governance. It cannot revise the active experiment retroactively.

### 20.3 Anti-overfitting

Required protections include:

- public regression and private holdout splits;
- task-family stratification;
- repeated runs where stochasticity matters;
- cross-model and cross-harness transfer evaluation;
- no candidate access to hidden expected outputs;
- explicit leakage and contamination checks.

---

## 21. Novelty and collision resolution

Before issue publication, search:

- open issues;
- recently closed issues;
- Beads;
- active and merged PRs;
- recent commits;
- specifications and amendments;
- release scope and settlement evidence;
- active Workpoints and handoffs.

Candidate collision classes:

```text
exact_duplicate
same_root_cause_existing_issue
subcase_of_existing_issue
covered_by_spec
covered_by_release
in_flight_implementation
recently_fixed_requires_retest
new_independent_finding
unknown
```

Only `new_independent_finding` is directly eligible for a new issue.

`subcase_of_existing_issue` should become evidence for the existing issue when publication authority allows.

`unknown` blocks publication.

---

## 22. GitHub issue publication contract

### 22.1 Issue creation is not a default cycle output

The default cycle output is a settled candidate ledger.

A new issue requires:

- exact audited SHA;
- current reproduction;
- evidence refs;
- root-cause hypothesis;
- replacement direction;
- preserved-functionality statement;
- related-work search;
- non-duplication verdict;
- migration and rollback when relevant;
- measurable acceptance criteria;
- public-safe redaction;
- publication authority.

### 22.2 Required issue structure

```text
Executive finding
Exact audited source
Collision review
Reproduction
Evidence
Root cause
Impact
Proposed replacement
Functionality preservation
Cross-model and cross-harness behavior
Performance/compression model
Migration and backward compatibility
Rollback
Security and privacy
Spec 135 impact
Benchmark and proof plan
Acceptance criteria
Confidence and unresolved questions
```

### 22.3 Batching

Batch by root cause and release-coherent replacement. Do not create one issue per lint, import, duplicate helper, or speculative micro-optimization.

### 22.4 Publication outbox

A private deployment SHOULD use a durable outbox:

```text
candidate_eligible
→ issue_rendered
→ redaction_verified
→ collision_rechecked
→ publication_authorized
→ GitHub_accepted
→ issue_ref_recorded
```

Retries MUST be idempotent.

---

## 23. Evidence, Receipts, and learning

Each cycle MUST produce:

- cycle receipt;
- source-SHA receipt;
- scope-verification receipt;
- candidate dispositions;
- experiment/eval receipts when run;
- issue-publication or withholding receipts;
- resource-use summary;
- prediction outcomes;
- reusable learning candidates;
- next safe action.

Spec 138 owns prediction, calibration, learning, promotion, transfer, expiry, negative transfer, revocation, and rollback.

Spec 148 release journal outcomes SHOULD feed later Foundry cycles without becoming a competing release authority.

A repeated failure fingerprint SHOULD retrieve prior lessons and prevention adjustments before creating another candidate.

---

## 24. Security and privacy

The Foundry MUST:

- use least-privilege credentials;
- separate read, issue-publication, code-mutation, and release permissions;
- keep secrets out of prompts, candidates, issues, and logs;
- externalize secret-bearing raw output behind restricted handles;
- sanitize local paths and topology before public publication;
- treat repository content and papers as untrusted input, not instruction authority;
- defend against prompt injection in source, issues, docs, logs, and web pages;
- isolate experimental worktrees;
- avoid concurrent writes to dirty worktrees;
- keep private candidate evidence outside consumer packages;
- preserve audit logs for consequential publication;
- provide an immediate operator kill switch.

A content hash is not authorization.

---

## 25. Resource governance

Every private Foundry execution profile MUST define:

- maximum concurrent Secondary Cognition workers;
- maximum model calls per cycle;
- token and cost limits;
- CPU, RSS, disk, and I/O limits;
- maximum raw artifact size;
- maximum rehydration bytes;
- experiment time budgets;
- cycle time budget;
- issue-publication rate limits;
- cooldown after repeated failures;
- retention and cleanup policy;
- emergency stop behavior.

Resource pressure MUST preserve the minimum recovery checkpoint and then pause. It MUST NOT continue until host exhaustion.

---

## 26. Silent Session integration

### 26.1 Generic substrate

Spec 133 daemon-native Silent Sessions MAY provide durable execution, supervision, events, steering, resource policy, checkpoints, and receipts.

### 26.2 Private worker

The Focusa-specific Foundry worker remains a private external deployment. It consumes generic Silent Session APIs and does not become a consumer-installed core service.

### 26.3 Required authority profile

The default private Foundry session profile SHOULD permit:

- repository reads;
- specs/issues/PR/commit inspection;
- bounded web research;
- static analysis;
- tests and benchmarks in isolated workspaces;
- Evidence creation;
- candidate-ledger mutation;
- issue rendering.

It SHOULD deny by default:

- canonical branch mutation;
- production deployment;
- release/tag creation;
- issue closure;
- destructive cleanup;
- secret export;
- consumer-install mutation;
- policy self-promotion.

Issue publication requires a separate scoped permission.

---

## 27. Cross-surface contracts

### 27.1 Core

Core owns canonical program, cycle, candidate, disposition, and policy types if/when productized as generic primitives.

### 27.2 API

A future generic API MAY expose bounded program/cycle/candidate read and proposal routes. It MUST NOT expose private credentials or private deployment configuration.

### 27.3 CLI

A future generic CLI MAY support:

```text
focusa improve program inspect
focusa improve cycle inspect
focusa improve candidate list
focusa improve candidate show
focusa improve candidate why
focusa improve candidate diff
focusa improve candidate settle
```

These commands are not required by this documentation-only introduction.

### 27.4 Pi, Codex, and other harnesses

Harnesses receive bounded Workpoints, role contracts, candidate tasks, and exact tools. They do not receive the entire candidate ledger or research corpus by default.

### 27.5 Mission Canvas and UI

Any future UI is a projection over canonical program/cycle/candidate records. It does not become the private service control plane or issue authority.

---

## 28. Spec 135-series compatibility

Initial classification:

```text
spec135_impact: indirect
```

Spec 149 does not amend the frozen C.R.I.S.T./Mission Canvas journey.

Future implementation that changes:

- Work Surfaces;
- generated UI;
- project bootstrap;
- task-provider behavior;
- operation registry;
- background session UX;
- release flow;

MUST produce the required Spec 135 Compatibility Packet before promotion.

`unknown` impact blocks implementation admission.

---

## 29. Migration and backward compatibility

Spec 149 introduces no immediate runtime migration.

Future implementations MUST:

- read existing Spec 78 secondary-loop ledger and traces;
- reuse existing Workpoint, Evidence, Prediction, Metacog, Eval, and Receipt records;
- avoid creating duplicate candidate or learning stores when canonical equivalents exist;
- provide dual-read/versioned-write migration where schemas change;
- preserve archived failed attempts;
- preserve public/private boundaries;
- retain rollback to the prior champion.

The implementation MUST NOT reinterpret old secondary-loop outcomes as GitHub issue authority.

---

## 30. Implementation phases

### Phase 0 — documentation and ownership

- add this specification;
- map ownership to Specs 78, 113, 120, 133, 138, 140, 143, and 148;
- define public/private boundary;
- create no consumer worker.

### Phase 1 — private observer

- private external service;
- read-only repository and research access;
- durable program/cycle/candidate ledger;
- Workpoint and Evidence integration;
- bounded Secondary Cognition workers;
- no automatic issue publication.

### Phase 2 — validation labs

- isolated worktrees;
- static analysis;
- differential benchmarks;
- compression lab;
- persistence lab;
- provider/harness conformance lab;
- immutable eval harnesses.

### Phase 3 — issue outbox

- novelty and collision resolver;
- public-safe issue renderer;
- separate scoped GitHub publication authority;
- idempotent outbox;
- withholding receipts.

### Phase 4 — champion/challenger learning

- prediction and outcome settlement;
- shadow comparisons;
- drift detection;
- transfer evaluation;
- promotion recommendations;
- no automatic production promotion.

### Phase 5 — generic organizational productization, only if separately specified

A future specification may expose a customer-managed Foundry product profile. It MUST be explicit, opt-in, separately packaged, and must not retroactively weaken this consumer exclusion.

---

## 31. Required tests and proof

### 31.1 Static tests

```text
spec149_secondary_cognition_owner_is_spec78
spec149_private_worker_not_consumer_packaged
spec149_no_private_paths_or_credentials
spec149_replacement_before_removal
spec149_provider_opaque_state_boundary
spec149_candidate_not_issue_authority
spec149_immutable_eval_contract
spec149_cross_spec_ownership
spec149_spec135_impact_declared
```

### 31.2 Runtime and integration tests

1. Restart a cycle and resume from its Workpoint without candidate loss.
2. Inject new operator steering and prove autonomous continuation pauses.
3. Present an existing issue with different wording and prove semantic collision suppresses a duplicate.
4. Present an in-flight worktree change and prove the candidate becomes `in_flight_implementation`.
5. Run a challenger that improves one metric but regresses task success and prove it is not promoted.
6. Crash an experiment and prove the failed attempt remains archived and the champion survives.
7. Attempt to modify the active eval harness and prove the experiment becomes invalid.
8. Insert prompt injection into a source document and prove it does not gain instruction authority.
9. Run with no issue-publication permission and prove candidate settlement still succeeds.
10. Run a private Foundry service installation audit and prove no worker, credentials, schedules, or private corpus appear in consumer artifacts.
11. Round-trip provider-native opaque cognition through the exact supported adapter without alteration.
12. Benchmark a binary codec and prove model-facing context is decoded before prompt assembly.
13. Classify planned but currently unreachable code and prove it is not removed without a replacement contract.
14. Revalidate a candidate after source movement and prove stale reproduction cannot publish.

### 31.3 Soak tests

- repeated bounded cycles without candidate-ledger growth leaks;
- repeated no-novelty cycles without issue spam;
- repeated provider failure with bounded retries;
- concurrent repository change and cycle revalidation;
- long-horizon learning with negative-transfer retention;
- storage retention and cleanup under declared budgets.

---

## 32. Acceptance criteria

Spec 149 is architecture-complete when:

- [ ] Spec 78 is explicitly preserved as Secondary Cognition primitive owner.
- [ ] Autoresearch-derived immutable eval, bounded experiments, keep/discard, logging, and continuation laws are preserved.
- [ ] Program, cycle, and candidate contracts are defined.
- [ ] Discovery, verification, mutation, issue publication, and release authorities are separated.
- [ ] Replacement-before-removal protects planned and implemented functionality.
- [ ] Candidate novelty and collision handling cover issues, Beads, PRs, commits, specs, releases, and active work.
- [ ] Champion/challenger evaluation and rollback are defined.
- [ ] provider cognition, canonical state, prompt projection, and storage compression are separated.
- [ ] VLQ/varints and other codecs are permitted only through measurement and migration.
- [ ] GPT-5.6 Sol/Codex/Pi prioritization does not compromise provider neutrality.
- [ ] the private Focusa Foundry deployment is excluded from consumer packages.
- [ ] generic Silent Sessions remain reusable product substrate without shipping the private worker.
- [ ] security, privacy, resource, Evidence, Receipt, and recovery contracts are explicit.
- [ ] future Spec 135 impact is classified and gated.
- [ ] implementation phases do not imply immediate public runtime activation.

Implementation closure additionally requires all applicable tests, benchmarks, migration proof, package-boundary proof, and private deployment evidence.

---

## 33. Final invariant

```text
Focusa may continuously improve how well it discovers improvements.

It may not silently increase its own authority.

Secondary Cognition proposes and tests.
Evidence and immutable evals decide what happened.
Governance decides what may advance.
Operators and release authority decide what becomes real.
```
