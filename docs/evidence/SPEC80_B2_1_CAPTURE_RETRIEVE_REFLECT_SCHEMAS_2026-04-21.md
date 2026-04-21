# SPEC80 B2.1 — Capture/Retrieve/Reflect Schema Finalization

Date: 2026-04-21
Bead: `focusa-yro7.2.2.1`
Labels: `planned-extension` (metacognition endpoints), `implemented-now` reference (`/v1/reflect/*` route family exists)

Purpose: finalize contract schemas and budget controls for:
- `focusa_metacog_capture`
- `focusa_metacog_retrieve`
- `focusa_metacog_reflect`

## 1) `focusa_metacog_capture`

### Input
```json
{
  "kind": "decision|constraint|failure|strategy|outcome",
  "content": "string",
  "rationale": "string?",
  "confidence": 0.0,
  "strategy_class": "diagnostic|corrective|preventive|exploratory",
  "clt_node_id": "string",
  "evidence_refs": ["docs/...", "crates/...", "tests/..."],
  "ontology_alignment": {
    "status": "active|speculative|verified|deprecated",
    "provenance_class": "user_asserted|tool_derived|model_inferred|verification_confirmed",
    "verification_state": "unverified|verified|rejected"
  }
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "capture_id": "string",
    "stored": true,
    "linked_turn_id": "string",
    "indexed_at": "ISO-8601"
  }
}
```

### Error codes
- `CAPTURE_SCHEMA_INVALID`
- `CAPTURE_POLICY_DENIED`

## 2) `focusa_metacog_retrieve`

### Input
```json
{
  "current_ask": "string",
  "scope_tags": ["string"],
  "k": 5,
  "budget": {
    "max_candidates": 20,
    "max_tokens": 1800,
    "max_latency_ms": 1200
  }
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "candidates": [
      {
        "capture_id": "string",
        "summary": "string",
        "score": 0.0,
        "rank": 1,
        "evidence_refs": []
      }
    ],
    "ranked_by": "hybrid_similarity",
    "retrieval_budget": {
      "tokens_used": 0,
      "latency_ms": 0,
      "truncated": false
    }
  }
}
```

### Error codes
- `RETRIEVE_UNAVAILABLE`
- `RETRIEVE_BUDGET_EXCEEDED`
- `RETRIEVE_INPUT_INVALID`

## 3) `focusa_metacog_reflect`

### Input
```json
{
  "turn_range": {"start": 0, "end": 0},
  "failure_classes": ["state_drift", "scope_miss", "evidence_gap"],
  "candidate_ids": ["string"],
  "budget": {
    "max_tokens": 2200,
    "max_latency_ms": 1800
  }
}
```

### Success output
```json
{
  "ok": true,
  "data": {
    "reflection_id": "string",
    "hypotheses": ["string"],
    "strategy_updates": ["string"],
    "confidence": 0.0,
    "next_action_hints": ["string"]
  }
}
```

### Error codes
- `REFLECT_INPUT_INVALID`
- `REFLECT_BUDGET_EXCEEDED`

## 4) Binding + permissions

- `focusa_metacog_capture`: `POST /v1/metacognition/capture` (planned), permission `metacognition:write`
- `focusa_metacog_retrieve`: `POST /v1/metacognition/retrieve` (planned), permission `metacognition:read`
- `focusa_metacog_reflect`: `POST /v1/metacognition/reflect` (planned), permission `metacognition:write`
- Existing route family anchor: `/v1/reflect/run|history|status` in `crates/focusa-api/src/routes/reflection.rs`

## 5) Budget invariants

1. Retrieval must hard-stop when `max_tokens` or `max_latency_ms` budget is hit.
2. Reflect must return explicit budget-exceeded error, not partial silent truncation.
3. Returned candidates must include ranking metadata (`score`, `rank`).

## 6) Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§6.2, §14.2, §15, §20.2)
- crates/focusa-api/src/routes/reflection.rs
- docs/24-capabilities-cli.md
