# Query Scope and Relevance Control

## Purpose

Define how Focusa prevents scope contamination, adjacent-thread leakage, and answer drift when a new user question arrives.

This document exists to prevent failures where the agent answers:
- a neighboring question
- a previously active thread
- a semantically adjacent topic
- an over-broadened interpretation

instead of the exact question actually asked.

---

## Core Thesis

Strong reasoning is not enough.

A competent agent must also preserve **question purity**.

That means it must explicitly model:
- what the current ask is
- what context is relevant
- what nearby context must be excluded
- what subject boundaries the answer must remain inside

Without this layer, memory and ontology can make the agent more knowledgeable but also more distortable.

---

## Design Laws

1. Every new user question should be treated as freshly scoped until relevance is proven.
2. Adjacent prior context must not be imported by default.
3. Relevance is earned, not assumed.
4. The current ask must be represented explicitly.
5. Exclusion of irrelevant prior context must be modeled explicitly.
6. Scope failures must be traceable and classifiable.

---

# 1. Core Object Types

## CurrentAsk
Represents the exact question, request, or instruction that should govern the current answer.

### Required properties
- `id`
- `ask_text`
- `ask_kind`
- `status`

---

## QueryScope
Represents the allowed subject boundary for the current answer.

### Required properties
- `id`
- `scope_kind`
- `status`

### Optional properties
- `allowed_topics`
- `excluded_topics`
- `carryover_policy`

---

## RelevantContextSet
Represents the subset of prior state that is actually relevant to the current ask.

### Required properties
- `id`
- `selection_kind`
- `status`

---

## ExcludedContextSet
Represents nearby prior state that must be kept out of the current answer.

### Required properties
- `id`
- `exclusion_kind`
- `status`

---

## ScopeFailure
Represents a failure where the answer escaped the proper scope.

### Required properties
- `id`
- `failure_kind`
- `severity`
- `status`

### Failure kinds
- `scope_contamination`
- `adjacent_thread_leakage`
- `answer_broadening`
- `wrong_question_answered`
- `context_overcarry`

---

# 2. Core Relation Types

## governed_by
Source:
- QueryScope

Target:
- CurrentAsk

---

## includes_context
Source:
- RelevantContextSet

Target:
- CurrentAsk
- Decision
- Constraint
- WorkingSet
- EvidenceArtifact
- VisualArtifact

---

## excludes_context
Source:
- ExcludedContextSet

Target:
- CurrentAsk
- Decision
- Constraint
- WorkingSet
- EvidenceArtifact
- VisualArtifact

---

## violates_scope_of
Source:
- ScopeFailure

Target:
- CurrentAsk
- QueryScope

---

# 3. Core Action Types

## determine_current_ask
Identify the exact question or request that should govern the answer.

## build_query_scope
Construct the allowed subject boundary.

## select_relevant_context
Select prior context that is actually relevant.

## exclude_irrelevant_context
Mark nearby context that must not carry over.

## verify_answer_scope
Check whether a proposed answer remains inside the current ask.

## record_scope_failure
Persist a scope contamination or wrong-question failure.

---

# 4. Guardrail Rules

Before answering, the agent should explicitly decide:
1. What is the exact question?
2. What prior context is relevant?
3. What prior context is adjacent but irrelevant?
4. What topics are out of scope for the answer?

If the answer relies on carryover from a prior thread, that carryover should be justified by relevance, not proximity.

---

# 5. Trace Requirements

Focusa should eventually trace:
- `current_ask_determined`
- `query_scope_built`
- `relevant_context_selected`
- `irrelevant_context_excluded`
- `scope_verified`
- `scope_failure_recorded`

---

# 6. Success Condition

Query Scope and Relevance Control is successful when Focusa helps an agent answer the exact question asked without letting adjacent prior context distort, broaden, or replace the current subject.
