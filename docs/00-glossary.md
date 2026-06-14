# docs/00-glossary.md — Focusa Canonical Glossary (LOCKED)

> **This glossary is authoritative.**  
> All documentation, code comments, agent instructions, and UI language MUST conform to the terms defined here.  
> No component may redefine these terms locally.
>
> See [`docs/current/AUTHORITY_MODEL.md`](current/AUTHORITY_MODEL.md) for the canonical authority role of each vocabulary-bearing surface.

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

## Runtime and Authority Terms

### Workpoint

A **Workpoint** is the canonical immediate continuation contract for active work: mission, current action, target objects, verified evidence, blockers, and next action. It survives compaction/model switches and overrides transcript tail for continuation. It is not a Beads replacement or a project goal by itself.

### ProjectIdentity

**ProjectIdentity** is the verified project boundary used to scope Focusa state and prevent cross-project drift. Canonical fields include `project_root`, `project_id`, canonical project name, and repo/workspace evidence.

### Continuity ID

A **Continuity ID** is the stable logical workstream identifier for a project/session lineage. It is not a Pi session id or timestamp.

### Session ID

A **Session ID** identifies a temporal agent/runtime session. It is metadata; authority comes from project scope plus continuity, not from session id alone.

### Canonical / Advisory / Degraded

**Canonical** means authoritative for verified scope. **Advisory** means useful for orientation but not authority. **Degraded** means partial, stale, fallback, or missing required authority. Never treat advisory/degraded output as canonical continuation truth.

### Evidence Ref

An **Evidence Ref** is a stable handle proving a result without raw transcript/log blobs. Examples: commit SHA, CI run id, test command, UIAI diagnostics id, API route proof.

### Focusa Daemon

The **Focusa Daemon** is the long-lived local/server runtime that hosts Focusa state and HTTP APIs. It is not the CLI or Mac app.

### Focusa CLI

The **Focusa CLI** is the operator command surface named `focusa`. It starts/stops or talks to the daemon and runs release, doctor, project, trajectory, bridge, and Workpoint commands.

### Menubar App

The **Menubar App** is the Mac ambient UI for Focusa awareness and connection status. It may store the VPS token; the phone does not become the permanent client in the Phone Bridge Flow.

---

## Phone Bridge Flow

**Definition**  
The **Phone Bridge Flow** is the temporary phone-mediated flow that connects the Mac Menubar App to Focusa on a VPS/server.

**Flow**
1. VPS/server runs `focusa pair`.
2. Phone opens the Focusa Connect Page from the VPS/server URL.
3. Mac Menubar App shows a Mac handoff QR.
4. Phone scans the Mac handoff QR.
5. Operator approves on the phone.
6. Mac stores the VPS/server URL and token.
7. Phone is done.

**Canonical Naming**
- Use **Phone Bridge Flow** for the whole flow.
- Use **phone bridge** for the temporary role the phone plays.
- Avoid **phone pairing** unless explicitly describing a permanent phone device credential, which this flow does not create.

**Related Terms**
- **Focusa Connect Page** — temporary browser/PWA page opened on the phone; knows server URL from origin and sends approval to the VPS/server.
- **Bridge Room** — short-lived server-side room created for the Phone Bridge Flow. Older docs/code may say **Pairing Room**; treat that as the implementation room concept, not user-facing flow naming.
- **Mac Handoff Offer** — short-lived QR payload shown by the Mac app; contains Mac/device name, nonce, and optional public-key/callback fields. It is not a token or server URL authority.
- **Mac Completion Payload** — fallback handoff data that lets the Mac store approved server URL/token when automatic callback/deeplink delivery is unavailable.
- **Mac Callback** — automatic return channel: the Mac app starts a local HTTP listener and embeds its address in the Mac Handoff Offer. After operator approval on the phone, the Focusa Connect Page POSTs the Mac Completion Payload to that address. No manual copy-paste required.
- **Public Focusa URL** — phone-reachable URL serving Focusa Connect and proxying required `/v1/connect/*` routes to the daemon.
- **Local Daemon URL** — loopback endpoint, usually `http://127.0.0.1:8787`; valid for local/dev use but not phone-reachable from another device.

**Install Contract**
Installers should write the Public Focusa URL to `/etc/focusa/public-url` or set `FOCUSA_PAIRING_URL`.

**What It Is Not**
- Not permanent phone pairing.
- Not a phone client install requirement.
- Not a long-lived phone token.

---

## Release Terms

### Release Stamp

A **Release Stamp** is the single tag-driven version update across Focusa CLI, daemon/API, core/TUI, Menubar App, Tauri metadata, lockfiles, and UI version display. Release version changes must be generated by the release stamper, not manually edited spot-by-spot.

### CI Tracking

**CI Tracking** means the release/tag script waits for GitHub CI and Release workflows and treats failures as incomplete release work. Pushing a tag is not completion; green CI and Release workflows are required proof.

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

## Trajectory Hierarchy

**Definition**
The **Trajectory Hierarchy** is the project-orientation ladder that steers work from the ultimate project direction down to executable slices.

**Canonical Acronyms**
- **HLT** — **High-Level Trajectory**: the ultimate project direction/north star. It describes what the project is trying to become.
- **MLG** — **Mid-Level Goal**: an intermediate milestone derived from the HLT. MLGs group related STGs and keep multi-step progress aligned.
- **STG** — **Short-Term Goal**: the immediate goal derived from the HLT through the current MLG/context. STGs guide the next bounded work slice.
- **Waypoint** — a concrete progress marker or checkpoint along an MLG/STG path. Waypoints help agents know where they are and what proof remains.

**Derivation Rule**

```text
HLT → MLG → STG → Waypoints → Workpoint
```

**Authority Rule**
- HLT, MLGs, STGs, and Waypoints steer orientation.
- Once a model knows the HLT, it must proactively plan toward the HLT instead of passively waiting or reacting turn-by-turn, except at explicit risk/approval gates.
- The model must defer to operator authority while actively offering HLT-aligned Waypoints, STGs, and MLGs as optional route guidance along the way.
- Workpoints remain the canonical immediate continuation contract.
- Evidence proves waypoint/STG progress.
- Operator steering wins over all trajectory projections.

**What It Is Not**
- Not autonomous task authority.
- Not a replacement for Beads.
- Not a reason to merge sessions without project_root + continuity_id match.

**HLT Ledger (append-only persistence)**
- Per Spec98/99: HLT changes are persisted to an append-only JSONL ledger scoped by `(project_root, continuity_id)`
- File: `{data_dir}/hlt-ledger/{project_root_hash}/hlt.jsonl`
- Each entry: timestamp, event_id, lamport_ts, project_root, continuity_id, session_id, old_hlt, new_hlt, source, reason, evidence_refs
- Tool: `focusa_hlt_history` exposes exact HLT history with old/new values
- Ledger is **append-only** — old entries are never modified or deleted
- See: `docs/102-trajectory-ladder-consolidated-spec.md §4.2`

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

---

## Context Cognition

**Definition**  
**Context Cognition** is an advisory bounded context packet that describes selected context, excluded context, scope, authority posture, freshness, evidence frame, reasoning frame, optimization frame, and route frame for the current project/workstream.

**Role**
- Helps agents understand relevant context without raw transcript dumps
- Supports bounded context curation and eval-backed promotion
- Exposes advisory/degraded/stale/mismatch state
- References source and rehydrate handles instead of becoming proof

**Authority**
- Advisory by default
- Never mutates Focus State
- Never replaces Workpoint
- Never overrides operator steering
- Must not be treated as canonical continuation truth

**What It Is Not**
- Not a Workpoint
- Not proof
- Not a transcript-tail authority source
- Not an autonomous reasoning optimizer claim

---

## Call Stack Design

**Definition**  
**Call Stack Design** is a typed, append-only implementation blueprint that describes the expected execution path for a feature before implementation.

**Canonical Shape**

```text
entry → handlers → services → adapters → storage → output
```

**Role**
- Makes implementation structure explicit before code changes
- Can link to a Workpoint when explicitly attached
- Can become evidence only when explicitly captured/linked
- Supports future implementation-drift verification

**Authority**
- Advisory by default
- Evidence-linkable when explicitly attached
- Never silently mutates Workpoint or Trajectory

**What It Is Not**
- Not executable authority
- Not a hidden task mutation
- Not a replacement for Workpoint, Context Authority, or evidence
