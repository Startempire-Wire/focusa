# docs/35-skill-to-capabilities-mapping.md — Agent Skills → Capabilities API (AUTHORITATIVE)

This document defines the **exact mapping** between the Focusa Agent Skill Bundle
and the Focusa Capabilities API.

Skills are **thin, declarative wrappers** over the Capabilities API.
They do not add logic.
They do not mutate state directly.
They do not bypass gates or policy.

---

## 0. Canonical Rule

> **Skills call capabilities.  
> Capabilities enforce policy.  
> Reducers enact cognition.**

---

## 1. Cognition Inspection Skills (Read-Only)

### `focusa.get_focus_state`

| Aspect | Value |
|------|------|
| Skill Type | Read |
| Capability | `state:read` |
| API Call | `GET /v1/state/current` |
| Side Effects | None |
| Telemetry | `skill.invoked`, `state.read` |

---

### `focusa.get_focus_stack`

| Aspect | Value |
|------|------|
| Capability | `state:read` |
| API Call | `GET /v1/state/stack` |
| Returns | Ordered focus frames |

---

### `focusa.get_lineage_tree`

| Aspect | Value |
|------|------|
| Capability | `lineage:read` |
| API Call | `GET /v1/lineage/tree` |
| Notes | Entire CLT (may be paginated) |

---

### `focusa.get_lineage_node`

| Aspect | Value |
|------|------|
| Capability | `lineage:read` |
| API Call | `GET /v1/lineage/node/{clt_id}` |
| Use | Drill-down explanations |

---

### `focusa.get_gate_explanation`

| Aspect | Value |
|------|------|
| Capability | `gate:read` |
| API Call | `GET /v1/gate/explain` |
| Use | Why focus changed or didn’t |

---

### `focusa.get_salient_references`

| Aspect | Value |
|------|------|
| Capability | `references:read` |
| API Call | `GET /v1/references/salient` |
| Notes | Gate-filtered |

---

### `focusa.get_constitution`

| Aspect | Value |
|------|------|
| Capability | `constitution:read` |
| API Call | `GET /v1/constitution/current` |
| Versioned | Yes |

---

### `focusa.get_autonomy_status`

| Aspect | Value |
|------|------|
| Capability | `autonomy:read` |
| API Call | `GET /v1/autonomy/status` |
| Includes | Constraints |

---

## 2. Telemetry & Metrics Skills (Read-Only)

### `focusa.get_token_stats`

| Aspect | Value |
|------|------|
| Capability | `telemetry:read` |
| API Call | `GET /v1/telemetry/tokens` |
| Filters | model, agent, session |

---

### `focusa.get_cognitive_metrics`

| Aspect | Value |
|------|------|
| Capability | `telemetry:read` |
| API Call | `GET /v1/telemetry/process` |
| Includes | focus depth, drift |

---

### `focusa.get_tool_usage`

| Aspect | Value |
|------|------|
| Capability | `telemetry:read` |
| API Call | `GET /v1/telemetry/tools` |
| Notes | Aggregated |

---

### `focusa.get_ux_signals`

| Aspect | Value |
|------|------|
| Capability | `telemetry:read` |
| API Call | `GET /v1/telemetry/ux` |
| Includes | UXP + UFI + evidence |

---

## 3. Explanation & Traceability Skills

### `focusa.explain_last_decision`

| Aspect | Value |
|------|------|
| Capability | `state:read` |
| API Call | `GET /v1/state/explain?scope=last` |
| Output | Gate + focus rationale |

---

### `focusa.trace_reference_usage`

| Aspect | Value |
|------|------|
| Capability | `references:read` |
| API Call | `GET /v1/references/trace/{ref_id}` |
| Use | Provenance |

---

## 4. Proposal & Request Skills (Guarded)

These skills **never enact change**.
They submit a **Command Request**, which is evaluated by:
- permissions
- Focus Gate
- autonomy level
- user approval

---

### `focusa.propose_focus_change`

| Aspect | Value |
|------|------|
| Capability | `commands:request` |
| API Call | `POST /v1/commands/request` |
| Payload |
```json
{
  "type": "focus.change",
  "params": {
    "new_focus": "...",
    "reason": "..."
  }
}
```

---

### `focusa.request_cache_bust`

| Aspect | Value |
|------|------|
| Capability | `commands:request` |
| API Call | `POST /v1/commands/request` |
| Payload |
```json
{
  "type": "cache.bust",
  "params": {
    "reason": "..."
  }
}
```

---

### `focusa.propose_constitution_update`

| Aspect | Value |
|------|------|
| Capability | `constitution:propose` |
| API Call | `POST /v1/constitution/propose` |
| Notes | Draft only |

---

### `focusa.request_command`

| Aspect | Value |
|------|------|
| Capability | `commands:request` |
| API Call | `POST /v1/commands/request` |
| Use | Export, diagnostics, etc. |

---

## 5. Response Contract (All Skills)

Every skill response MUST include:

```json
{
  "status": "success | rejected | pending",
  "data": { ... },
  "explanation": "string",
  "citations": [
    { "type": "clt", "id": "uuid" },
    { "type": "telemetry", "id": "uuid" }
  ]
}
```

No silent failures.
No ambiguous outcomes.

---

## 6. Telemetry Hooks (Mandatory)

Every skill invocation emits:

- `skill.invoked`
- `capability.accessed`
- `permission.checked`
- `result.returned`

These are recorded by CTL.

---

## 7. Security Guarantees

- Skills cannot bypass permission checks
- Skills cannot mutate reducers
- Skills cannot alter Focus State directly
- Skills are scoped by agent identity
- Skills respect autonomy level

---

## 8. Canonical Rule

> **If a skill cannot be mapped to a capability, it must not exist.**

---

## 9. Strategic Outcome

This mapping ensures:

- agents reason from truth
- system state is never hallucinated
- autonomy grows safely
- Focusa remains deterministic
- platform extensibility remains controlled

This completes the **agent ↔ Focusa contract**.
