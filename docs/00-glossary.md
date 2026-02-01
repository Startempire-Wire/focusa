# docs/00-glossary.md — Focusa Canonical Glossary (LOCKED)

> **This glossary is authoritative.**  
> All documentation, code comments, agent instructions, and UI language MUST conform to the terms defined here.  
> No component may redefine these terms locally.

---

## Focusa

**Definition**  
Focusa is a local cognitive runtime that preserves focus, intent, and meaning across long-running AI sessions by separating cognition from conversation.

**What Focusa Is**
- A focus and context operating layer
- Harness-agnostic
- Local-first
- Deterministic
- Human-aligned

**What Focusa Is Not**
- Not a model
- Not an agent framework
- Not an automation engine
- Not a RAG system
- Not a scheduler

---

## Focus State

**Definition**  
The **Focus State** represents the system’s current state of mind: what it is doing, why it is doing it, what has been decided, and what must remain true.

**Role**
- Primary carrier of meaning across turns
- Survives context compaction
- Injected into every model invocation

**Typical Contents**
- Intent
- Decisions
- Constraints
- Artifacts (by reference)
- Failures
- Next steps

**What It Is Not**
- Not a chat summary
- Not raw conversation history
- Not inferred memory

---

## Focus Stack

**Definition**  
The **Focus Stack** is a hierarchical structure that organizes Focus States into nested frames of attention.

**Role**
- Models human task nesting
- Ensures only one active focus at a time
- Enables clean suspension and resumption of work

**Properties**
- Exactly one active Focus Frame
- Parent frames contribute selectively
- Completed frames are archived, not forgotten

**What It Is Not**
- Not a conversation log
- Not a call stack for code execution

---

## Focus Frame

**Definition**  
A **Focus Frame** is a single unit of focused work within the Focus Stack, bound to a concrete intent (typically a Beads issue).

**Required Properties**
- Title
- Goal
- Bound Beads issue ID
- Focus State
- Completion reason (when closed)

**What It Is Not**
- Not a chat turn
- Not speculative thinking
- Not multi-tasking

---

## Focus Gate

**Definition**  
The **Focus Gate** is the conscious filter that determines which potential concerns are allowed to surface into awareness.

**Role**
- Receives signals from the Intuition Engine
- Applies decay, pressure, and pinning rules
- Surfaces candidates for human or agent review

**Key Property**
- Advisory only — never auto-switches focus

**What It Is Not**
- Not a decision engine
- Not an interrupt system
- Not autonomous

---

## Intuition Engine

**Definition**  
The **Intuition Engine** is the subconscious processing layer that detects patterns, anomalies, repetition, and weak signals below awareness.

**Role**
- Runs asynchronously
- Observes without acting
- Aggregates signals over time
- Feeds Focus Gate

**Examples of Signals**
- Repeated errors
- Long-running tasks
- Inconsistencies
- Blockers
- Time-based pressure

**What It Is Not**
- Not reasoning
- Not planning
- Not orchestration
- Not decision-making

---

## Reference Store

**Definition**  
The **Reference Store** is the externalized memory system that holds large or durable artifacts outside the prompt.

**Role**
- Prevents token overload
- Preserves lossless artifacts
- Enables explicit rehydration

**Examples**
- File diffs
- Logs
- Tool outputs
- Test results

**Key Property**
- Referenced by handles, not inlined

**What It Is Not**
- Not semantic memory
- Not inferred knowledge
- Not automatically injected

---

## Expression Engine

**Definition**  
The **Expression Engine** converts the current Focus State into language suitable for model invocation.

**Role**
- Selects what to say *now*
- Enforces token budgets
- Applies deterministic structure
- Handles explicit degradation

**Key Property**
- Deterministic and bounded

**What It Is Not**
- Not reasoning
- Not planning
- Not summarization for memory

---

## Beads

**Definition**  
**Beads** is the authoritative task and long-term intent memory system used by Focusa.

**Role**
- Stores tasks, dependencies, and progress
- Governs what work exists
- Provides durable planning memory

**Key Property**
- If work is not in Beads, it does not exist

---

## Session

**Definition**  
A **Session** is an isolated execution context representing a single continuous Focusa run.

**Role**
- Prevents cross-contamination of state
- Scopes Focus Stack, Reference Store, and memory

**Key Property**
- All state mutations must belong to a Session

---

## Candidate

**Definition**  
A **Candidate** is a potential concern surfaced by the Intuition Engine and evaluated by the Focus Gate.

**Properties**
- Pressure
- Source
- Age
- Pinned flag

**What It Is Not**
- Not an action
- Not a command
- Not a decision

---

## Memory (Semantic / Procedural)

**Definition**  
Memory in Focusa is small, explicit, and user-approved.

**Types**
- Semantic: facts, preferences
- Procedural: rules, habits

**Key Property**
- Never inferred automatically

---

## Pinning

**Definition**  
Pinning marks an item as resistant to decay and eligible for continued relevance.

**Applicable To**
- Focus Gate candidates
- Focus State sections
- Reference Store artifacts
- Memory entries

**What It Is Not**
- Not priority override
- Not automation

---

## Non-Goals (Global)

The following are explicitly out of scope for Focusa:

- Autonomous task execution
- Model training or RL
- Kernel-level attention optimization
- Hidden prompt manipulation
- Silent memory mutation
- Cloud dependency

---

## Canonical Cognitive Flow

```
Intuition Engine
      ↓
  Focus Gate
      ↓
 Focus Stack
      ↓
 Focus State
      ↓
Expression Engine
      ↓
  Model Invocation
```

---

## Final Invariant

> **Meaning lives in Focus State, not in conversation.**

This invariant underpins all design decisions in Focusa.
