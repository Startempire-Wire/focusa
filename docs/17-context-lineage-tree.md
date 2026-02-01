# docs/17-context-lineage-tree.md — Context Lineage Tree (CLT) Specification (AUTHORITATIVE)

The **Context Lineage Tree (CLT)** is a persistent, append-only, tree-structured
representation of *interaction history and compaction lineage*.

CLT exists to preserve **structural context, branching history, and compaction
traceability** without contaminating canonical cognition.

CLT is **not memory**, **not focus**, and **not authority**.

---

## 1. Purpose

The CLT answers one question:

> “What interaction paths existed, which were followed, which were abandoned,
and how were they compacted over time?”

It does **not** answer:
- what the system believes
- what the current goal is
- what should be done next

Those belong to Focus State and the reducer.

---

## 2. Core Design Rules (Non-Negotiable)

1. CLT is **append-only**
2. Nodes are **immutable once written**
3. CLT never mutates Focus State
4. Focus State references **exactly one CLT node** as its lineage head
5. Compaction inserts nodes — it never deletes history
6. Branches may be abandoned, summarized, but never erased
7. CLT is inspectable, navigable, and replayable

---

## 3. Conceptual Position in the Architecture

```
Raw Events / Messages
        │
        ▼
┌─────────────────────────┐
│ Context Lineage Tree    │  ← structural history & branching
│ (CLT)                   │
└────────────▲────────────┘
             │
┌────────────┴────────────┐
│ Focus State (Canonical) │  ← single authoritative cognition
└─────────────────────────┘
```

---

## 4. CLT Node Model

Every CLT entry is a **node** with a parent pointer.

### 4.1 Canonical Node Shape

```json
{
  "node_id": "clt_000124",
  "node_type": "interaction | summary | branch_marker",

  "parent_id": "clt_000118",
  "created_at": "2025-02-18T13:44:10Z",

  "session_id": "session_42",

  "payload": { },
  "metadata": { }
}
```

- `parent_id = null` indicates the root
- Only one node per session is the **current head**

---

## 5. Node Types

### 5.1 Interaction Node

Represents a single interaction or atomic step.

```json
{
  "node_type": "interaction",
  "payload": {
    "role": "user | assistant | system",
    "content_ref": "ref://artifact/abc123"
  },
  "metadata": {
    "task_id": "beads-124",
    "agent_id": "focusa-default",
    "model_id": "claude-3.5"
  }
}
```

**Important**
- CLT does not store raw text
- Content lives in the Reference Store
- CLT stores only handles

---

### 5.2 Summary Node (Compaction)

Represents a **structured summary of an abandoned or compacted path**.

```json
{
  "node_type": "summary",
  "payload": {
    "summary_type": "abandoned_path | compaction",
    "summary_ref": "ref://artifact/summary_91af"
  },
  "metadata": {
    "covers_range": ["clt_000112", "clt_000118"],
    "reason": "context_compaction"
  }
}
```

Summary nodes:
- collapse multiple prior nodes conceptually
- preserve lineage
- remain inspectable

---

### 5.3 Branch Marker Node

Explicitly marks a branching decision.

```json
{
  "node_type": "branch_marker",
  "payload": {
    "branch_reason": "user_rephrase | alternative_strategy",
    "label": "retry_with_constraints"
  },
  "metadata": {
    "initiator": "user | agent"
  }
}
```

Branch markers make divergence explicit and analyzable.

---

## 6. Focus State Integration (Critical)

Focus State MUST reference **exactly one CLT node**:

```json
{
  "focus_state": {
    "active_frame_id": "frame_7",
    "lineage_head": "clt_000124"
  }
}
```

Rules:
- Focus State always advances the CLT head
- Switching focus does not mutate CLT
- CLT does not select focus

---

## 7. Compaction Rules

When compaction occurs:

1. Identify a contiguous path segment
2. Generate a structured summary
3. Insert a `summary` node
4. Reattach the active head to the summary node
5. Preserve original nodes as ancestors

Nothing is deleted.

---

## 8. Branching Rules

Branching:
- creates a new node whose parent is an earlier node
- does not require file duplication
- allows multiple abandoned futures

Only one branch is ever “active” via Focus State.

---

## 9. Complexity Guarantees

- Append: O(1)
- Branch: O(1)
- Context reconstruction: O(depth)
- No linear scans required

---

## 10. Relationship to Other Systems

| System | Interaction |
|---|---|
| Focus State | References CLT head |
| Reducer | Emits CLT nodes (never reads entire tree) |
| Reference Store | Stores content referenced by CLT |
| Intuition Engine | Observes patterns (read-only) |
| CS | Consumes summaries & branch history |
| UFI | Links friction signals to CLT nodes |
| UI | Visualizes tree & navigation |

---

## 11. What CLT Is NOT

- Not memory
- Not belief
- Not task state
- Not conversation history
- Not authority

CLT is **lineage**, not cognition.

---

## 12. Observability & Tooling

CLT MUST support:
- branch navigation
- abandoned path inspection
- summary expansion
- timeline replay
- visual tree rendering

All without mutating state.

---

## 13. Canonical Rule (Write This Everywhere)

> **The Context Lineage Tree preserves where we have been,  
> not what we currently believe.**

---

## 14. Why This Exists (Final Note)

CLT enables:
- aggressive compaction without amnesia
- safe experimentation
- explainable abandonment
- long-horizon autonomy
- constitutional evolution with evidence

Without sacrificing determinism.
