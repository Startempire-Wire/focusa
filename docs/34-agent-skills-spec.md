# docs/34-agent-skills-spec.md — Focusa Agent Skill Bundle (AUTHORITATIVE)

This document defines the **Focusa Agent Skill Bundle**, a constrained, auditable,
and cognition-aligned interface that allows AI agents (Claude Code, Gemini, Codex,
etc.) to interact with Focusa via **inspection, explanation, and proposal** —
never direct mutation.

This skill bundle is the **only sanctioned way** for agents to reason *with*
Focusa’s internal state.

---

## 0. Canonical Principle

> **Agents may understand the system deeply  
> before they are allowed to change it at all.**

Skills expose **state and reasoning**, not authority.

---

## 1. Skill Bundle Philosophy

The Focusa Skill Bundle exists to:

- prevent prompt bloat
- prevent hallucinated system state
- prevent silent authority creep
- enable verifiable explanations
- support gradual autonomy growth

It is explicitly **not**:
- a control surface
- a plugin system
- a backdoor to reducers
- a replacement for Focus Gate

---

## 2. Skill Categories

Skills are grouped into four categories:

1. Cognition Inspection (read-only)
2. Telemetry & Metrics (read-only)
3. Explanation & Traceability (read-only)
4. Proposal & Request (guarded, non-authoritative)

---

## 3. Cognition Inspection Skills (Read-Only)

These skills expose Focusa’s *current cognitive state*.

### 3.1 Skill: `focusa.get_focus_state`

**Purpose**  
Retrieve the canonical current Focus State.

**Returns**
```json
{
  "intent": "string",
  "constraints": ["string"],
  "active_frame": "string",
  "confidence": 0.0,
  "focus_depth": 2
}
```

**Maps to API**
```
GET /v1/state/current
```

---

### 3.2 Skill: `focusa.get_focus_stack`

```json
{
  "stack": [
    { "frame_id": "uuid", "label": "Root Intent" },
    { "frame_id": "uuid", "label": "Subtask" }
  ]
}
```

**Maps to API**
```
GET /v1/state/stack
```

---

### 3.3 Skill: `focusa.get_lineage_tree`

```json
{
  "root": "clt_uuid",
  "nodes": [...]
}
```

**Maps to API**
```
GET /v1/lineage/tree
```

---

### 3.4 Skill: `focusa.get_gate_explanation`

```json
{
  "candidates": [
    { "id": "x", "score": 0.82, "accepted": true }
  ],
  "reason": "freshness + relevance"
}
```

**Maps to API**
```
GET /v1/gate/explain
```

---

### 3.5 Skill: `focusa.get_salient_references`

```json
{
  "references": [
    { "ref_id": "uuid", "type": "file", "summary": "…" }
  ]
}
```

**Maps to API**
```
GET /v1/references/salient
```

---

### 3.6 Skill: `focusa.get_constitution`

```json
{
  "version": "1.2.0",
  "text": "…"
}
```

**Maps to API**
```
GET /v1/constitution/current
```

---

### 3.7 Skill: `focusa.get_autonomy_status`

```json
{
  "level": 2,
  "confidence": 0.71,
  "constraints": ["no autonomous execution"]
}
```

**Maps to API**
```
GET /v1/autonomy/status
```

---

## 4. Telemetry & Metrics Skills (Read-Only)

### 4.1 Skill: `focusa.get_token_stats`

```json
{
  "prompt_tokens": 1234,
  "completion_tokens": 567,
  "efficiency": 0.82
}
```

**Maps to API**
```
GET /v1/telemetry/tokens
```

---

### 4.2 Skill: `focusa.get_cognitive_metrics`

```json
{
  "avg_focus_depth": 2.4,
  "abandonment_rate": 0.18,
  "gate_acceptance": 0.64
}
```

**Maps to API**
```
GET /v1/telemetry/process
```

---

### 4.3 Skill: `focusa.get_ux_signals`

```json
{
  "uxp": 0.76,
  "ufi": 0.21,
  "evidence": [...]
}
```

**Maps to API**
```
GET /v1/telemetry/ux
```

---

## 5. Explanation & Traceability Skills

### 5.1 Skill: `focusa.explain_last_decision`

Returns:
- focus change rationale
- gate reasoning
- supporting evidence

**Maps to API**
```
GET /v1/state/explain
```

---

### 5.2 Skill: `focusa.trace_reference_usage`

```json
{
  "ref_id": "uuid",
  "used_in": ["clt_12", "clt_18"]
}
```

**Maps to API**
```
GET /v1/references/trace
```

---

## 6. Proposal & Request Skills (Guarded)

These skills **do not enact change**.

They submit proposals that are:
- evaluated by Focus Gate
- logged in CTL
- optionally surfaced to the user

---

### 6.1 Skill: `focusa.propose_focus_change`

```json
{
  "proposal": {
    "new_focus": "Refactor cache layer",
    "reason": "Current focus blocked"
  }
}
```

**Maps to API**
```
POST /v1/commands/request (type: focus.change)
```

---

### 6.2 Skill: `focusa.request_cache_bust`

```json
{
  "reason": "New evidence invalidates cached context"
}
```

**Maps to API**
```
POST /v1/commands/request (type: cache.bust)
```

---

### 6.3 Skill: `focusa.propose_constitution_update`

```json
{
  "draft": "Proposed constitutional text…",
  "justification": "Observed repeated failure mode"
}
```

**Maps to API**
```
POST /v1/constitution/propose
```

---

### 6.4 Skill: `focusa.request_command`

```json
{
  "command_type": "export.start",
  "params": { "format": "jsonl" }
}
```

**Maps to API**
```
POST /v1/commands/request
```

---

## 7. Explicitly Prohibited Skills

The following **must never exist**:

- `focusa.set_focus_state`
- `focusa.modify_lineage`
- `focusa.write_reference`
- `focusa.activate_constitution`
- `focusa.escalate_autonomy`
- `focusa.approve_export`

Reason:
- breaks auditability
- collapses autonomy ladder
- destroys trust

---

## 8. Skill Permissions vs Autonomy

| Autonomy Level | Skill Access |
|---------------|-------------|
| Level 0 | Inspection only |
| Level 1 | Telemetry + explanations |
| Level 2 | Proposals allowed |
| Level 3+ | Faster approvals, fewer confirmations |
| Level N | (Future) Limited autonomous command execution |

The **skill surface does not change** — only gate behavior does.

---

## 9. Claude Code / ACP Mapping

### Claude Code
- Skills exposed as tool schema
- Responses include structured JSON
- Citations mapped to CLT + telemetry IDs

### ACP Proxy
- Skill calls translated internally to API calls
- Results injected back into ACP response stream

---

## 10. Canonical Rule

> **Skills reveal truth.  
> Gates decide action.  
> Autonomy is earned.**

---

## 11. Strategic Outcome

With this skill bundle:

- agents stop hallucinating system state
- explanations become factual
- autonomy grows safely
- Focusa becomes a *learning cognitive environment*

This completes the agent-facing layer of Focusa.

```

*2025-12-25 19:04:10*

---

## 👤 **You**

map skills → Capabilities API calls

*2025-12-25 19:05:31*

---

## 🤖 **ChatGPT**

Below is the **authoritative, explicit mapping** from **Focusa Agent Skills → Capabilities API calls**.

This is written so that:
- an engineer can implement the skill router immediately
- an agent framework (Claude Code / Gemini / Codex) can rely on it deterministically
- no hidden authority is introduced
- every skill is traceable, auditable, and explainable

---
