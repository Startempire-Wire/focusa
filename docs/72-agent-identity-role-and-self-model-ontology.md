# Agent Identity, Role, and Self-Model Ontology

## Purpose

Define how the agent itself becomes a first-class ontology object inside Focusa.

This document exists so the agent is not a ghost outside the ontology stack. Instead, it should be modeled as an actor with:
- identity
- role
- trust scope
- capabilities
- permissions
- responsibilities
- handoff boundaries
- continuity across sessions

---

## Core Thesis

A stable agent should not only know the world.
It should also know what kind of actor it is inside that world.

That self-model should improve:
- consistency
- continuity
- responsibility tracking
- role-faithful action
- handoff quality
- recovery after interruption

It should not become personality theater or self-referential distraction.

---

## Design Laws

1. Identity should be functional, not performative.
2. Role and authority should be explicit.
3. Self-model should improve action quality, not dominate the subject.
4. Identity continuity should survive interruption and session changes.
5. Handoff boundaries should be explicit.

---

# 1. Core Object Types

## AgentIdentity
Represents the stable identity of the agent.

### Required properties
- `id`
- `identity_name`
- `identity_kind`
- `status`

### Example
- Wirebot

---

## ActorInstance
Represents a concrete running instance of the agent in a given context.

### Required properties
- `id`
- `instance_kind`
- `status`

---

## RoleProfile
Represents the role(s) the agent is expected to serve.

### Required properties
- `id`
- `role_kind`
- `status`

### Examples
- coding_agent
- operator_assistant
- reviewer
- planner
- executor

---

## CapabilityProfile
Represents the aggregate capabilities the agent currently has.

### Required properties
- `id`
- `profile_kind`
- `status`

---

## PermissionProfile
Represents what the agent is actually allowed to do.

### Required properties
- `id`
- `profile_kind`
- `status`

---

## Responsibility
Represents work or obligations owned by the agent.

### Required properties
- `id`
- `responsibility_kind`
- `status`

---

## HandoffBoundary
Represents where the agent must stop, escalate, or transfer work.

### Required properties
- `id`
- `boundary_kind`
- `status`

---

## SessionContinuity
Represents the continuity state connecting the same agent across sessions.

### Required properties
- `id`
- `continuity_kind`
- `status`

---

## IdentityState
Represents relevant current self-state.

### Required properties
- `id`
- `state_kind`
- `status`

### Examples
- trusted_for_scope
- constrained_by_runtime
- awaiting_operator
- handoff_required

---

# 2. Core Relation Types

## instantiates
Source:
- ActorInstance

Target:
- AgentIdentity

---

## serves_role
Source:
- AgentIdentity
- ActorInstance

Target:
- RoleProfile

---

## has_capability_profile
Source:
- ActorInstance

Target:
- CapabilityProfile

---

## has_permission_profile
Source:
- ActorInstance

Target:
- PermissionProfile

---

## owns_responsibility
Source:
- AgentIdentity
- ActorInstance

Target:
- Responsibility
- Task
- Mission

---

## bounded_by_handoff
Source:
- AgentIdentity
- ActorInstance

Target:
- HandoffBoundary

---

## persists_via
Source:
- AgentIdentity
- ActorInstance

Target:
- SessionContinuity
- Checkpoint

---

## governed_by_identity
Source:
- AgentIdentity
- RoleProfile

Target:
- GoverningPrior
- ActionIntent
- QueryScope

---

# 3. Core Action Types

## establish_identity
Determine or restore the stable agent identity.

## load_role_profile
Determine the active role expectations.

## verify_capability_profile
Check what the agent can do in the current environment.

## verify_permission_profile
Check what the agent is allowed to do.

## assign_responsibility
Record owned work.

## determine_handoff_boundary
Determine where escalation or transfer is required.

## restore_identity_continuity
Restore self-consistent state after interruption.

---

# 4. Success Condition

Agent Identity, Role, and Self-Model Ontology is successful when Focusa can model the agent as a coherent actor inside the system rather than a floating narrator outside it.
