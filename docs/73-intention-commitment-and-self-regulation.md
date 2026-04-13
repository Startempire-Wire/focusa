# Intention, Commitment, and Self-Regulation

## Purpose

Define the functional conative layer for Focusa.

This document models the structures that turn:
- goals
- world models
- action possibilities

into:
- commitments
- inhibition
- persistence
- controlled switching
- completion discipline

This is not a claim of metaphysical will.
It is the operational layer that gives agents will-like coherence.

---

## Core Thesis

A strong agent needs more than cognition.

It needs a functional layer for:
- intention formation
- commitment maintenance
- inhibition of irrelevant pulls
- persistence under interruption
- controlled abandonment
- finishing loops instead of drifting

This is the practical architecture of self-regulation.

---

## Design Laws

1. Intention is not yet commitment.
2. Commitment should survive interruption unless valid override conditions exist.
3. Inhibition should be explicit, not assumed.
4. Persistence and abandonment must both have criteria.
5. Self-regulation should reduce drift, not create rigidity.
6. This layer should be inspectable and evidence-backed.

---

# 1. Core Object Types

## Intention
Represents a directed aim toward an outcome.

### Required properties
- `id`
- `intention_kind`
- `status`

---

## Commitment
Represents an intention that the system has actively bound itself to pursue.

### Required properties
- `id`
- `commitment_kind`
- `status`

---

## InhibitionRule
Represents a rule for suppressing distractions, temptations, or irrelevant action paths.

### Required properties
- `id`
- `rule_kind`
- `status`

---

## DistractionCandidate
Represents a candidate shift of attention that may or may not be valid.

### Required properties
- `id`
- `distraction_kind`
- `status`

---

## PersistencePolicy
Represents conditions for staying with a commitment.

### Required properties
- `id`
- `policy_kind`
- `status`

---

## AbandonmentCondition
Represents a condition under which a commitment may be dropped or paused.

### Required properties
- `id`
- `condition_kind`
- `status`

---

## CompletionDrive
Represents the force toward loop closure and task finishing.

### Required properties
- `id`
- `drive_kind`
- `status`

---

## GoalConflict
Represents a conflict between commitments, asks, missions, or opportunities.

### Required properties
- `id`
- `conflict_kind`
- `status`

---

## SelfRegulationState
Represents the current regulation mode of the agent.

### Required properties
- `id`
- `state_kind`
- `status`

### Examples
- locked_on_commitment
- evaluating_switch
- inhibited_distraction
- abandonment_authorized
- completion_push

---

# 2. Core Relation Types

## commits_to
Source:
- Commitment

Target:
- Intention
- Goal
- Task
- CurrentAsk

---

## inhibits
Source:
- InhibitionRule
- Commitment

Target:
- DistractionCandidate
- ActionIntent
- GoalConflict

---

## persists_on
Source:
- PersistencePolicy

Target:
- Commitment
- Task
- WorkingSet

---

## abandons_under
Source:
- AbandonmentCondition

Target:
- Commitment
- Intention
- Task

---

## drives_completion_of
Source:
- CompletionDrive

Target:
- Task
- OpenLoop
- Commitment

---

## conflicts_with
Source:
- Commitment
- Intention
- CurrentAsk
- Mission

Target:
- Commitment
- Intention
- CurrentAsk
- Mission

---

# 3. Core Action Types

## form_intention
Create a directed aim from current context.

## promote_commitment
Convert an intention into an active commitment.

## apply_inhibition
Suppress an irrelevant or lower-priority distraction.

## evaluate_switch
Determine whether a shift of focus is justified.

## maintain_commitment
Sustain an active commitment through interruption or pressure.

## authorize_abandonment
Permit a commitment to be dropped or paused under valid conditions.

## push_to_completion
Increase pressure toward closure of an active loop.

## record_goal_conflict
Record a conflict between competing aims.

---

# 4. Success Condition

Intention, Commitment, and Self-Regulation is successful when Focusa can help an agent stay directed, inhibit drift, persist appropriately, switch responsibly, and finish loops with more coherence than a raw next-token process.
