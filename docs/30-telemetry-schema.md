# docs/30-telemetry-schema.md — Telemetry Event Schema (AUTHORITATIVE)

This document defines the **canonical schema** for all telemetry events.

---

## 1. Base Event Envelope

```json
{
  "event_id": "uuid",
  "event_type": "string",
  "timestamp": "iso8601",
  "session_id": "uuid",
  "agent_id": "uuid",
  "model_id": "string",
  "focus_frame_id": "optional uuid",
  "clt_id": "optional uuid",
  "payload": { },
  "schema_version": "1.0"
}
```

---

## 2. Token Usage Event

```json
{
  "event_type": "model.tokens",
  "payload": {
    "prompt_tokens": 1234,
    "completion_tokens": 456,
    "cached_tokens": 890,
    "cache_hit": true,
    "latency_ms": 832,
    "provider": "anthropic",
    "model": "claude-3.5",
    "temperature": 0.2
  }
}
```

---

## 3. Focus Transition Event

```json
{
  "event_type": "focus.transition",
  "payload": {
    "from_frame": "uuid",
    "to_frame": "uuid",
    "reason": "gate.accepted",
    "depth": 3
  }
}
```

---

## 4. CLT Event

```json
{
  "event_type": "lineage.node.created",
  "payload": {
    "node_type": "interaction | summary | branch",
    "parent_id": "uuid",
    "summary": "optional"
  }
}
```

---

## 5. Gate Decision Event

```json
{
  "event_type": "gate.decision",
  "payload": {
    "candidates": 5,
    "accepted": 1,
    "scores": {
      "candidate_a": 0.92,
      "candidate_b": 0.41
    }
  }
}
```

---

## 6. Tool Invocation Event

```json
{
  "event_type": "tool.call",
  "payload": {
    "tool": "fs.read",
    "duration_ms": 120,
    "success": true,
    "output_refs": ["ref_uuid"]
  }
}
```

---

## 7. UXP / UFI Event

```json
{
  "event_type": "ux.signal",
  "payload": {
    "type": "satisfaction | frustration",
    "weight": 0.73,
    "evidence": [
      { "type": "explicit", "source": "rating" },
      { "type": "behavioral", "source": "override" }
    ]
  }
}
```

---

## 8. Autonomy Event

```json
{
  "event_type": "autonomy.update",
  "payload": {
    "previous_level": 2,
    "new_level": 3,
    "confidence": 0.84,
    "reason": "sustained_success"
  }
}
```

---

## 9. Schema Versioning

- Every event includes `schema_version`
- New fields are additive only
- No breaking changes in minor versions

---

## 10. Canonical Rule

> **Events are facts. Metrics are interpretations.**
