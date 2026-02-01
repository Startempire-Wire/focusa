# docs/20-training-dataset-schema.md — Focusa Training Dataset Schema (AUTHORITATIVE)

This document defines the canonical schemas for exporting Focusa data into
training datasets suitable for:

- Supervised Fine-Tuning (SFT)
- Preference Optimization (DPO / IPO)
- Failure-aware contrastive training
- Long-horizon reasoning discipline
- Autonomy curriculum learning

The schema is **model-agnostic**, **license-aware**, and **lineage-preserving**.

---

## 0. Canonical Principle

> **Training data must represent cognition, not conversation.**

All exported samples MUST be:
- grounded in Focus State
- traceable to CLT lineage
- labeled with outcome signals (UXP/UFI)
- stripped of provider fingerprints

---

## 1. Dataset Families

Focusa exports data into **four primary dataset families**:

1. `focusa_sft`
2. `focusa_preference`
3. `focusa_contrastive`
4. `focusa_long_horizon`

Each family has a strict schema.

---

## 2. Common Fields (ALL DATASETS)

Every record MUST include the following fields:

```json
{
  "record_id": "uuid",
  "session_id": "session_uuid",
  "agent_id": "agent_uuid",
  "agent_constitution_version": "semver",
  "model_id": "string | null",
  "harness_id": "string | null",

  "focus_state_id": "focus_state_uuid",
  "focus_state_snapshot": { },

  "clt_head": "clt_node_id",
  "clt_path": ["clt_node_id", "..."],

  "task_context": {
    "task_system": "beads | other | none",
    "task_id": "string | null",
    "task_label": "string | null"
  },

  "uxp": {
    "score": 0.0,
    "confidence": 0.0
  },
  "ufi": {
    "score": 0.0,
    "confidence": 0.0
  },

  "autonomy": {
    "level": 0,
    "earned_score": 0.0
  },

  "license": {
    "export_allowed": true,
    "consent_version": "string"
  },

  "timestamps": {
    "started_at": "iso8601",
    "completed_at": "iso8601"
  }
}
```

---

## 3. Focus State Snapshot (Canonical Shape)

Focus State is exported **structurally**, never as prose.

```json
{
  "intent": "string",
  "constraints": ["string"],
  "active_focus_frame": "string",
  "focus_stack_depth": 2,
  "salient_references": ["ref://artifact/..."],
  "excluded_context": ["string"],
  "confidence": 0.0
}
```

---

## 4. Dataset Family Schemas

---

### 4.1 `focusa_sft` — Supervised Fine-Tuning

**Use case**
- Instruction tuning
- Stable, high-confidence behaviors

**Eligibility**
- UXP ≥ threshold
- UFI ≤ threshold
- Task completed successfully

**Schema Extension**
```json
{
  "instruction": "string",
  "context": {
    "references": ["ref://artifact/..."],
    "summaries": ["ref://artifact/..."]
  },
  "response": "string",
  "response_metadata": {
    "token_count": 1234,
    "format": "markdown | text | json"
  }
}
```

---

### 4.2 `focusa_preference` — Preference / DPO

**Use case**
- Preference learning
- Calibration improvement
- Alignment without RL

**Schema Extension**
```json
{
  "prompt": "string",
  "response_a": "string",
  "response_b": "string",
  "preferred": "a | b",
  "preference_basis": {
    "uxp_delta": 0.32,
    "ufi_delta": -0.18,
    "user_corrections": 2
  }
}
```

Preferences are inferred — never guessed.

---

### 4.3 `focusa_contrastive` — Failure-aware Training

**Use case**
- Teaching models what *not* to do
- Drift resistance

**Schema Extension**
```json
{
  "goal": "string",
  "failed_path": {
    "summary": "string",
    "clt_nodes": ["clt_id", "..."]
  },
  "successful_path": {
    "summary": "string",
    "clt_nodes": ["clt_id", "..."]
  },
  "failure_reason": [
    "stale_context",
    "constraint_violation",
    "wrong_focus",
    "tool_misuse"
  ]
}
```

---

### 4.4 `focusa_long_horizon` — Procedural / Temporal Reasoning

**Use case**
- Long-session coherence
- Multi-step planning discipline

**Schema Extension**
```json
{
  "episode": {
    "initial_intent": "string",
    "state_transitions": [
      {
        "focus_state_delta": { },
        "action_taken": "string",
        "outcome": "string"
      }
    ],
    "final_outcome": "success | failure | partial"
  }
}
```

No chain-of-thought is leaked — only structured transitions.

---

## 5. Reference Resolution Rules

- `ref://artifact/*` paths must be resolvable
- Artifacts may be exported inline OR as sidecar files
- Dataset consumers must not require live Focusa runtime

---

## 6. Decontamination Requirements

Before export:
- Strip provider phrasing
- Remove system messages
- Normalize tool output formats
- Exclude eval prompts
- Exclude cached responses reused across contexts

---

## 7. Output Formats

Supported:
- JSONL (default)
- Parquet (optional)
- HuggingFace `datasets` compatible

---

## 8. Canonical Rule

> **If provenance, focus, or outcome cannot be proven — do not export.**
