# Affordance and Execution-Environment Ontology

## Purpose

Define the ontology layer that models what can actually be done, by whom, under what conditions, at what cost, with what risk, and with what reversibility.

This document exists to close the gap between:
- cognition
- world models
- real action

Focusa already models:
- goals, tasks, decisions, constraints, risks, blockers, working sets
- software and UI worlds

What this layer adds is the ontology of **practical possibility**.

It answers questions like:
- What can be done right now?
- What is blocked by missing authority, missing tools, missing resources, or missing prerequisites?
- Which action path is cheapest, safest, fastest, or most reversible?
- Which agent or actor is allowed to do what?
- What must be unlocked before a plan becomes executable?

This layer is domain-general and should support software, UI, operations, research, business processes, and future real-world agent environments.

---

## Core Thesis

A competent agent needs more than:
- a goal
- a world model
- an intended action

It also needs a model of:
- available affordances
- execution surfaces
- permissions and authority boundaries
- resource availability
- dependency chains
- action cost and latency
- action reliability and reversibility

Without that layer, Focusa can know what should happen next but still be weak at choosing what is actually feasible and safe.

---

## Design Laws

1. Affordance modeling must be domain-general.
2. Capability and permission are not the same thing.
3. Preconditions and dependencies must be explicit.
4. Cost, latency, reliability, and reversibility must be first-class.
5. Authority boundaries must be modelable.
6. Execution-environment truth should be grounded in evidence where possible.
7. Affordance state should integrate with blockers, risks, and working sets.

---

# 1. Core Object Types

## Capability
Represents an abstract ability to perform a class of actions.

### Required properties
- `id`
- `capability_kind`
- `status`

### Examples
- read_repo
- write_repo
- deploy_service
- run_migration
- capture_screenshot
- critique_ui
- modify_schema

---

## ToolSurface
Represents a concrete action surface through which capability is exercised.

### Required properties
- `id`
- `surface_kind`
- `status`

### Examples
- CLI command
- HTTP route
- editor integration
- GitHub write surface
- daemon command
- external API

---

## Permission
Represents a granted or denied authorization state.

### Required properties
- `id`
- `permission_kind`
- `status`

### Examples
- repo_write_allowed
- production_access_denied
- org_admin_required
- connector_access_granted

---

## AuthorityBoundary
Represents a boundary where action control changes hands.

### Required properties
- `id`
- `boundary_kind`
- `status`

### Examples
- operator approval required
- org admin approval required
- deployment gate
- payment authorization gate

---

## Precondition
Represents a condition that must hold before an action is executable.

### Required properties
- `id`
- `precondition_kind`
- `status`

### Examples
- dependency installed
- session active
- checkpoint created
- reference artifact exists
- migration reviewed

---

## Dependency
Represents an upstream requirement or external dependency.

### Required properties
- `id`
- `dependency_kind`
- `status`

### Examples
- package dependency
- service availability
- API key present
- connector enabled
- database reachable

---

## Resource
Represents a consumable or bounded resource.

### Required properties
- `id`
- `resource_kind`
- `status`

### Examples
- token budget
- memory budget
- compute budget
- time budget
- screen real estate
- storage budget

---

## CostModel
Represents the estimated or observed cost of using an affordance.

### Required properties
- `id`
- `cost_kind`
- `status`

### Examples
- token cost
- time cost
- money cost
- operator attention cost
- risk cost

---

## LatencyProfile
Represents expected or observed delay before an action yields useful results.

### Required properties
- `id`
- `latency_kind`
- `status`

### Examples
- immediate
- low_latency
- async_minutes
- async_hours

---

## ReliabilityProfile
Represents the observed or expected likelihood of success.

### Required properties
- `id`
- `reliability_kind`
- `status`

### Examples
- deterministic
- flaky
- environment_sensitive
- unverified
- high_confidence

---

## ReversibilityProfile
Represents how recoverable an action is after execution.

### Required properties
- `id`
- `reversibility_kind`
- `status`

### Examples
- fully_reversible
- compensating_action_possible
- partially_reversible
- irreversible

---

## Ownership
Represents who owns a system, resource, or decision surface.

### Required properties
- `id`
- `owner_kind`
- `status`

### Examples
- operator_owned
- org_owned
- team_owned
- external_vendor_owned

---

## ExecutionContext
Represents the current environment in which an action would execute.

### Required properties
- `id`
- `context_kind`
- `status`

### Examples
- local_dev
- staging
- production
- constrained_chat_runtime
- live_server_runtime
- offline_mode

---

## Affordance
Represents an actionable opportunity available in a specific context.

### Required properties
- `id`
- `affordance_kind`
- `status`

### Optional properties
- `recommended`
- `reason`
- `confidence`

### Examples
- safe_local_edit_available
- github_pr_available
- runtime_restart_available
- screenshot_capture_available
- deploy_blocked_by_authority

---

# 2. Core Relation Types

## enabled_by
Source:
- Capability
- Affordance

Target:
- ToolSurface
- ExecutionContext
- Resource

---

## requires_permission
Source:
- Capability
- ToolSurface
- Affordance

Target:
- Permission

---

## bounded_by_authority
Source:
- Capability
- ToolSurface
- Affordance

Target:
- AuthorityBoundary
- Ownership

---

## depends_on
Source:
- Capability
- ToolSurface
- Affordance
- Precondition

Target:
- Dependency
- Precondition
- Resource

---

## consumes_resource
Source:
- Capability
- ToolSurface
- Affordance

Target:
- Resource
- CostModel
- LatencyProfile

---

## has_reliability
Source:
- ToolSurface
- Affordance

Target:
- ReliabilityProfile

---

## has_reversibility
Source:
- Capability
- ToolSurface
- Affordance

Target:
- ReversibilityProfile

---

## available_in_context
Source:
- Capability
- ToolSurface
- Affordance

Target:
- ExecutionContext

---

## blocks_execution_of
Source:
- Permission
- AuthorityBoundary
- Dependency
- Precondition
- Resource

Target:
- Affordance
- Capability
- ActionIntent

---

## supports_execution_of
Source:
- Capability
- ToolSurface
- Affordance

Target:
- ActionIntent
- Task
n
---

# 3. Core Action Types

## detect_affordances
Infer what can be done in the current context.

## verify_permissions
Check whether the actor is actually authorized.

## verify_preconditions
Check whether the action is executable now.

## evaluate_dependencies
Check required upstream conditions and systems.

## estimate_cost
Estimate token/time/money/attention cost.

## estimate_latency
Estimate time-to-result.

## estimate_reliability
Estimate or score success likelihood.

## estimate_reversibility
Determine how recoverable the action is.

## choose_execution_path
Select the best action path from available affordances.

## escalate_authority
Identify when human/operator/admin approval is required.

## mark_unavailable
Record when an affordance is currently blocked or unavailable.

---

# 4. Evidence Types

## permission_evidence
Evidence that access is granted or denied.

## dependency_evidence
Evidence that a dependency exists, is missing, or is degraded.

## resource_evidence
Evidence about available or depleted resources.

## execution_evidence
Evidence from actual use of a tool surface.

## reliability_evidence
Observed success/failure history.

## reversibility_evidence
Evidence about rollback or recovery behavior.

## authority_evidence
Evidence about ownership, approvals, or approval requirements.

---

# 5. What This Enables

With this layer, Focusa can:
- distinguish between desired actions and feasible actions
- understand when it is blocked by permissions, dependencies, or environment
- choose safer or cheaper action paths
- know when escalation or approval is needed
- reason about reversibility before taking risky action
- integrate real-world execution constraints into planning and working-set construction

---

# 6. Success Condition

The Affordance and Execution-Environment Ontology is successful when Focusa can model practical possibility well enough to choose actions based not only on intent and world understanding, but also on what is truly executable, authorized, affordable, reliable, and reversible in the current environment.
