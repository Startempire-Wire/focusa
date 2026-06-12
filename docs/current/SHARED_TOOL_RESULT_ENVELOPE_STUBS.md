# Shared Tool Result Envelope Stubs

**Status:** implementation seed for `focusa-877z.8.3`.

All API, CLI, Pi, UIAI, MCP, and menubar surfaces should normalize status through `tool_result_v1` semantics before rendering or retry decisions.

## Required shared fields

```json
{
  "ok": true,
  "status": "completed",
  "canonical": true,
  "advisory": false,
  "degraded": false,
  "stale": false,
  "scope": {
    "project_root": "${FOCUSA_PROJECT_ROOT:-<focusa-repo>}",
    "continuity_id": "focusa-cont-example",
    "workpoint_id": "optional",
    "scope_status": "verified",
    "scope_source": "focusa_verified"
  },
  "failure_class": null,
  "summary": "bounded result summary",
  "retry": {"safe": true, "posture": "safe_retry", "reason": "optional"},
  "side_effects": [],
  "evidence_refs": [],
  "next_tools": ["focusa_workpoint_resume"],
  "recovery_hint": "optional",
  "misuse_hint": "optional"
}
```

## Surface stubs

### API route stub

```rust
json!({
  "details": {"tool_result_v1": {
    "ok": true,
    "status": "completed",
    "canonical": true,
    "advisory": false,
    "degraded": false,
    "stale": false,
    "scope": {"project_root": project_root, "continuity_id": continuity_id, "scope_status": "verified"},
    "failure_class": null,
    "summary": summary,
    "retry": {"safe": true, "posture": "safe_retry"},
    "side_effects": side_effects,
    "evidence_refs": evidence_refs,
    "next_tools": next_tools
  }}
})
```

### CLI JSON stub

```json
{
  "status": "completed",
  "tool_result_v1": {
    "ok": true,
    "canonical": true,
    "advisory": false,
    "degraded": false,
    "stale": false,
    "scope": {"scope_status": "verified"},
    "failure_class": null,
    "retry": {"safe": true, "posture": "safe_retry"},
    "side_effects": [],
    "evidence_refs": [],
    "next_tools": []
  }
}
```

### Pi tool wrapper stub

```ts
focusaToolResult({
  ok,
  status,
  canonical,
  advisory,
  degraded,
  stale,
  scope,
  summary,
  failure_class,
  retry,
  side_effects,
  evidence_refs,
  next_tools,
})
```

### UIAI packet bridge stub

UIAI `ResearchDiagnosticsPacket` is not a Focusa tool result, but its bridge/render must map proposal status into compatible fields:

```json
{
  "canonical": false,
  "advisory": true,
  "degraded": false,
  "stale": false,
  "scope": {"scope_status": "present", "scope_source": "caller_supplied"},
  "side_effects": ["external_io"],
  "evidence_refs": ["uiai-diagnostics:session=..."],
  "next_tools": ["focusa_browser_diagnostics_intake"]
}
```

### Menubar display stub

Menubar cards render these fields directly:

- `canonical=true` → solid authority chip.
- `advisory=true` → advisory/proposal chip.
- `degraded=true` or `stale=true` → warning chip and recovery hint.
- `scope.scope_status != verified` → scope warning and project verify route.
- `side_effects` and `evidence_refs` shown in proof peek.

## Status vocabulary

Allowed `status` values: `accepted`, `completed`, `pending`, `no_op`, `blocked`, `validation_rejected`, `degraded`, `offline`, `error`.

Allowed `scope.scope_status` values: `verified`, `present`, `missing`, `partial`, `mismatch_candidate`, `unsafe`, `unknown`.

Allowed `retry.posture` values: `safe_retry`, `retry_with_idempotency_key`, `check_side_effects_first`, `do_not_retry_unchanged`, `operator_required`.

## Rule

If a surface cannot fill a field, it must emit `canonical=false`, `degraded=true` or `advisory=true`, plus a recovery hint; it must not silently omit authority status.
