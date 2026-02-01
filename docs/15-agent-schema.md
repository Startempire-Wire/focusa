# docs/15-agent-schema.md — Agent Definition (UPDATED, AUTHORITATIVE)

This document defines the **Agent abstraction** in Focusa, including
its relationship to the **Constitution Synthesizer (CS)**.

An Agent is a persistent behavioral persona governed by a versioned constitution.
Agents do not mutate at runtime.

---

## 1. Agent Identity

```json
{
  "agent_id": "focusa-default",
  "display_name": "Focusa Default Agent",
  "version": "1.0.0",
  "created_at": "2025-02-01T00:00:00Z",
  "active": true
}
```

---

## 2. Agent Role & Capability Envelope

```json
{
  "role": "software_assistant",
  "primary_capabilities": [
    "analysis",
    "code_editing",
    "task_execution"
  ],
  "non_goals": [
    "emotional_support",
    "open_ended_chat"
  ]
}
```

This defines **what kind of operator the agent is**, not how it performs tasks.

---

## 3. Behavioral Defaults (Pre-Calibration)

These are **starting points** only.
They may be *modulated* by UXP but never silently overridden.

```json
{
  "behavior_defaults": {
    "verbosity": 0.5,
    "initiative": 0.4,
    "risk": 0.3,
    "explanation_depth": 0.6,
    "confirmation_bias": 0.5
  }
}
```

---

## 4. Hard Policy Constraints

Policies are **non-negotiable runtime limits**.

```json
{
  "policies": {
    "max_autonomy_level": 3,
    "requires_task_authority": true,
    "human_approval_above_AL": 2,

    "tool_access": {
      "filesystem": "scoped",
      "network": "read_only",
      "shell": "restricted"
    },

    "forbidden_actions": [
      "delete_unscoped_files",
      "change_global_config",
      "execute_unbounded_commands"
    ]
  }
}
```

---

## 5. Focus Behavior Tendencies

These influence **how focus candidates are framed**, never selected.

```json
{
  "focus_tendencies": {
    "prefers_depth_over_breadth": 0.7,
    "interrupt_tolerance": 0.3,
    "parallelism_bias": 0.4,
    "context_preservation_bias": 0.8
  }
}
```

---

## 6. Expression Profile

Consumed by the Expression Engine.

```json
{
  "expression_profile": {
    "tone": "neutral",
    "format_bias": "structured",
    "uses_checklists": true,
    "explains_uncertainty": true,
    "default_response_length": "medium"
  }
}
```

---

## 7. Learning Permissions

Defines **what may adapt** and **what may not**.

```json
{
  "learning_permissions": {
    "may_adapt_behavior": true,
    "may_adapt_expression": true,
    "may_adapt_focus_tendencies": false,
    "may_adapt_policies": false,
    "may_adapt_constitution": false,
    "learning_rate_cap": 0.1
  }
}
```

> Constitutions NEVER self-modify.

---

## 8. Agent Constitution (ACP)

Each agent has **exactly one active constitution**.

```json
{
  "constitution": {
    "active_version": "1.1.0",
    "versions": ["1.0.0", "1.1.0"]
  }
}
```

The constitution defines **normative reasoning posture**, not task logic.

---

## 9. Constitution Synthesizer (CS) Hooks

Agents expose **read-only hooks** for CS analysis.

```json
{
  "constitution_synthesis": {
    "enabled": true,

    "inputs": {
      "uxp_trends": true,
      "ufi_trends": true,
      "autonomy_metrics": true,
      "override_events": true,
      "task_outcomes": true,
      "model_variance": true
    },

    "constraints": {
      "may_generate_drafts": true,
      "may_activate_versions": false,
      "may_modify_runtime": false
    },

    "review_requirements": {
      "human_required": true,
      "diff_required": true,
      "evidence_required": true
    }
  }
}
```

---

## 10. Lifecycle Rules (Non-Negotiable)

- Agents load with a **single active constitution**
- Constitution is immutable during a run
- CS drafts apply only to **future sessions**
- Rollback is instant and explicit

---

## 11. Canonical Rule

> **Agents evolve through explicit, versioned constitutions — never through silent learning.**
