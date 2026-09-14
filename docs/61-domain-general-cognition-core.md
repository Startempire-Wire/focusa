# Domain-General Cognition Core

## Purpose

Define the smallest reusable cognition layer that Focusa can apply across domains, not only software and UI work.

This document exists to capture what makes a strong operator a strong thinker:
- decomposition
- constraint tracking
- decision formation
- risk handling
- blocker detection
- verification
- working-set maintenance
- recovery after interruption or failure
- finishing loops instead of drifting

This document is intentionally domain-general.

It does **not** define:
- software-world primitives
- UI-world primitives
- workflow phase sequencing
- target-system integrations

Those are layered on top of this core.

---

## Core Thesis

A good developer is not only a code specialist.

A good developer is a strong thinker operating inside a structured working world.

That same thinking structure can apply to:
- software building
- UI design
- research
- operations
- planning
- content systems
- other real-life processes

Focusa should therefore model a domain-general cognition layer that domain ontologies plug into.

---

## Design Laws

1. Keep the cognition layer domain-neutral.
2. Keep domain ontology layers separate from cognition primitives.
3. Make interruption and recovery first-class.
4. Treat verification as central, not optional.
5. Make blockers, risks, and open loops explicit.
6. Support both planning and execution.
7. Preserve bounded working sets instead of broad history dependence.

---

# 1. Core Object Types

## Mission
Represents the current overarching objective.

### Required properties
- `id`
- `title`
- `objective`
- `status`

---

## Goal
Represents a bounded objective within a mission.

### Required properties
- `id`
- `title`
- `status`

---

## Task
Represents a concrete unit of work.

### Required properties
- `id`
- `title`
- `status`
- `priority`

---

## Subtask
Represents a more granular work unit under a task.

### Required properties
- `id`
- `title`
- `status`

---

## Decision
Represents a durable conclusion that should affect future action.

### Required properties
- `id`
- `statement`
- `decision_kind`
- `status`

---

## Constraint
Represents a rule that should shape action selection.

### Required properties
- `id`
- `rule_text`
- `scope`
- `enforcement_level`
- `status`

---

## Risk
Represents a known threat or fragility.

### Required properties
- `id`
- `title`
- `severity`
- `status`

---

## Blocker
Represents a condition preventing progress.

### Required properties
- `id`
- `summary`
- `severity`
- `status`

---

## OpenLoop
Represents unresolved pending work, uncertainty, or incomplete closure.

### Required properties
- `id`
- `statement`
- `urgency`
- `status`

---

## WorkingSet
Represents the bounded current world required for competent action.

### Required properties
- `id`
- `working_set_kind`
- `status`

### Examples
- active_mission
- debugging
- design
- research
- execution
- recovery

---

## ActionIntent
Represents a proposed next action over the working world.

### Required properties
- `id`
- `action_kind`
- `status`

---

## Verification
Represents evidence that an assumption, action, or outcome is correct or incorrect.

### Required properties
- `id`
- `verification_kind`
- `result`
- `status`

---

## Checkpoint
Represents a resumable snapshot of operative state.

### Required properties
- `id`
- `checkpoint_kind`
- `status`

---

## EvidenceArtifact
Represents source material or output evidence.

### Required properties
- `id`
- `artifact_kind`
- `status`

---

# 2. Core Relation Types

## contains
Source:
- Mission
- Goal
- Task

Target:
- Goal
- Task
- Subtask

---

## constrains
Source:
- Constraint
- Decision

Target:
- Mission
- Goal
- Task
- ActionIntent
- WorkingSet

---

## blocks
Source:
- Blocker
- Risk
- OpenLoop

Target:
- Goal
- Task
- ActionIntent

---

## supports
Source:
- Task
- Subtask
- Decision
- Verification

Target:
- Goal
- Mission

---

## derived_from
Source:
- Decision
- Verification
- EvidenceArtifact
- Checkpoint

Target:
- EvidenceArtifact
- Task
- ActionIntent

---

## verifies
Source:
- Verification

Target:
- Decision
- Constraint
- Task
- ActionIntent
- Goal

---

## belongs_to_working_set
Source:
- Goal
- Task
- Decision
- Constraint
- Risk
- Blocker
- OpenLoop
- EvidenceArtifact

Target:
- WorkingSet

---

## supersedes
Source:
- Decision
- Checkpoint

Target:
- Decision
- Checkpoint

---

## transitions_to
Source:
- ActionIntent
- Task
- WorkingSet

Target:
- Task
- WorkingSet
- Checkpoint

---

# 3. Core Action Types

## decompose_goal
Break a goal into tasks or subtasks.

## prioritize_work
Order tasks and working-set members.

## record_decision
Promote a durable conclusion.

## register_constraint
Record a rule that should shape future action.

## identify_risk
Record a fragility or threat.

## mark_blocked
Record a blocker state.

## restore_progress
Resume from checkpoint after interruption.

## verify_progress
Check whether an assumption or outcome is actually true.

## refresh_working_set
Recompute the bounded current world.

## close_loop
Resolve or retire an open loop.

## complete_task
Mark a task complete with evidence.

---

# 4. Core Evidence Types

## source_input
Human/operator input or source material.

## observation
A structured fact or detected signal.

## result_artifact
An execution result or produced output.

## checkpoint_artifact
A resumable state snapshot.

## comparison_artifact
An artifact used to compare expected vs actual.

---

# 5. What This Enables

With this layer, Focusa can generalize beyond software because it can always represent:
- what we are trying to do
- what matters right now
- what constrains us
- what we decided
- what is risky
- what is blocking us
- what still needs closure
- what should happen next
- what proves we are right
- where to resume after interruption

---

# 6. Success Condition

The Domain-General Cognition Core is successful when Focusa can provide a reusable thinking substrate that domain ontologies plug into, allowing strong cognition patterns to transfer beyond software work.
