# Focusa

> **A local-first cognitive governance framework for AI agents.**
>
> Focusa is not a chatbot. Not an agent framework. Not a RAG system.
> It is the system that makes agents **trustworthy over time**.

---

## The Problem

Modern AI systems fail in long-running sessions because:

1. **Conversation is treated as memory.** When chat history is all you have, compaction silently destroys meaning, decisions vanish, and the model "forgets what it was doing."

2. **Intent drifts.** There is no structured notion of "current focus." Nested subtasks collapse into linear chat. Constraints and decisions stated 50 turns ago disappear.

3. **Priority confusion.** Everything is treated as equally important. No organic surfacing of what matters *now*. No way to say "this is background noise, that is critical."

4. **Trust is unverifiable.** Autonomy is granted by config flags, not earned through evidence. There is no way to measure whether an agent is improving, regressing, or drifting. Learning is invisible, identity is mutable, and rollback is impossible.

5. **Compaction destroys meaning.** Automatic summarization silently deletes: why a decision was made, what constraints exist, what artifacts were referenced, what the next step should be. The agent starts over from a lossy summary.

This is not a token problem. It is a **continuity of mind** problem.

## The Core Insight

> **Meaning should never live only in conversation.**

Focusa extracts, structures, and persists meaning *outside* the model so that context compaction never destroys intent. Focus State replaces chat history as the carrier of meaning. Compaction becomes harmless because everything that matters is already somewhere safe.

## What Focusa Is

Focusa is a **local cognitive proxy** that sits between an AI harness (Claude Code, Codex CLI, Gemini CLI, Letta, Zed ACP, or any OpenAI-compatible API) and the model backend. It governs **focus, context fidelity, and priority emergence** across long-running sessions.

Focusa does NOT replace agents, models, or frameworks. It augments them by providing:

- **Hierarchical focus control** — know exactly what the system is doing and why
- **Deterministic context compression** — meaning survives compaction
- **Lossless artifact offloading** — large data referenced by handle, not inlined
- **Advisory salience surfacing** — priorities emerge without auto-acting
- **Explicit, auditable memory** — no hidden writes, no personality drift
- **Earned autonomy** — trust measured by evidence, not granted by config
- **Constitutional governance** — agent identity evolves deliberately, never silently

### What Focusa Is Not

- Not a model
- Not an agent framework
- Not an automation engine
- Not a RAG system
- Not a scheduler
- Not autonomous
- Not cloud-dependent

## System Diagram

```
┌─ EXTERNAL ──────────────────────────────────────────────────────────────────────────────┐
│                                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
│  │  Claude Code  │  │  Codex CLI   │  │  Gemini CLI  │  │  Zed (ACP)   │  │  Letta   │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬─────┘ │
│         └──────────────────┴────────────┬────┴─────────────────┘               │       │
│                                         │                                       │       │
│                              ┌──────────▼──────────┐              ┌─────────────▼─────┐ │
│                              │   Proxy Adapter      │              │  ACP Proxy        │ │
│                              │   Mode A: CLI Wrap   │              │  Mode C: JSON-RPC │ │
│                              │   Mode B: HTTP Proxy │              │  p50<5ms p95<15ms │ │
│                              └──────────┬──────────┘              └─────────┬─────────┘ │
│                                         └────────────────┬──────────────────┘           │
│                                                          │                               │
│  User                                                    │   Prompt + Response            │
│  ──────────────────────────────────────────              │   (intercepted)                │
│  Override · Pin · Suppress · Grant                        │                               │
│  Autonomy · Edit Constitution · Fork                     │                               │
│  Thread · Rehydrate Artifact                              │                               │
│  ───────────────────────────────────┐                    │                               │
│                                     │                    │                               │
└─────────────────────────────────────┼────────────────────┼───────────────────────────────┘
                                      │                    │
┌─ FOCUSA DAEMON (Rust) ─────────────────────────────────────────────────────────────────────┐
│                                      │                    │                                  │
│  ┌───────────────────────────────────┼────────────────────┼────────────────────────────┐    │
│  │            ╔═══════════════════════════════════════════╗                             │    │
│  │            ║         CORE REDUCER (single writer)      ║                             │    │
│  │            ║                                           ║                             │    │
│  │            ║  reduce(state, event) → {new_state,       ║                             │    │
│  │            ║                          emitted_events}  ║                             │    │
│  │            ║                                           ║                             │    │
│  │            ║  15 event types · 7 invariants            ║                             │    │
│  │            ║  Deterministic · Replayable · No I/O      ║                             │    │
│  │            ╚═══════════╤═══════════════╤═══════════════╝                             │    │
│  │                        │               │                                              │    │
│  │            ┌───────────▼───┐   ┌───────▼──────────────────────────────────────┐      │    │
│  │            │  FocusaState  │   │  Emitted Events                              │      │    │
│  │            │  {            │   │  → Persistence (JSON + JSONL)                 │      │    │
│  │            │   session     │   │  → CTL Telemetry (append-only)                │      │    │
│  │            │   focus_stack │   │  → SSE Event Stream (CLI/TUI/GUI)             │      │    │
│  │            │   focus_gate  │   │  → Worker Queue (async jobs)                  │      │    │
│  │            │   ref_index   │   └──────────────────────────────────────────────┘      │    │
│  │            │   memory      │                                                          │    │
│  │            │   version     │                                                          │    │
│  │            │  }            │                                                          │    │
│  │            └───────────────┘                                                          │    │
│  │                                                                                       │    │
│  └───────────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                              │
│  ┌─ PLANE 1: COGNITIVE CONTROL ───────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌─────────────────────────────────┐     ┌──────────────────────────────────────────┐  │  │
│  │  │  FOCUS STACK (HEC)              │     │  FOCUS GATE (RAS-inspired)               │  │  │
│  │  │                                 │     │                                          │  │  │
│  │  │  Hierarchical Execution Context │     │  Pre-conscious salience filter           │  │  │
│  │  │                                 │     │                                          │  │  │
│  │  │  ┌─ Root Frame (paused) ──┐     │     │  Signals ──→ Candidates ──→ Surfaced     │  │  │
│  │  │  │  ┌─ Parent (paused) ─┐ │     │     │                                          │  │  │
│  │  │  │  │  ┌─ ACTIVE ─────┐ │ │     │     │  Pressure formula:                       │  │  │
│  │  │  │  │  │ • title      │ │ │     │     │    base (+0.2 to +2.0 by signal kind)    │  │  │
│  │  │  │  │  │ • goal       │ │ │     │     │    × goal alignment (×0.8/×1.1/×1.3)     │  │  │
│  │  │  │  │  │ • beads_id   │ │ │     │     │    + recency (+0.3 if <5min)              │  │  │
│  │  │  │  │  │ • ASCC ckpt  │ │ │     │     │    + risk (+0.4 if error/warning)         │  │  │
│  │  │  │  │  │ • ECS refs   │ │ │     │     │    × decay (×0.98 per tick)               │  │  │
│  │  │  │  │  │ • constraints│ │ │     │     │                                          │  │  │
│  │  │  │  │  └──────────────┘ │ │     │     │  Surface threshold: 2.2                  │  │  │
│  │  │  │  └───────────────────┘ │     │     │  Pinned items: immune to decay            │  │  │
│  │  │  └────────────────────────┘     │     │  Advisory only — NEVER auto-switches      │  │  │
│  │  │                                 │     │                                          │  │  │
│  │  │  Exactly 1 active frame         │     │  Candidates:                              │  │  │
│  │  │  Every frame → Beads issue      │     │   suggest_push_frame                      │  │  │
│  │  │  Completion reasons:            │     │   suggest_resume_frame                    │  │  │
│  │  │   goal_achieved | blocked       │     │   suggest_check_artifact                  │  │  │
│  │  │   abandoned | superseded        │     │   suggest_fix_error                       │  │  │
│  │  │   error                         │     │   suggest_pin_memory                      │  │  │
│  │  └─────────────────────────────────┘     └──────────────────────────────────────────┘  │  │
│  │                                                         ▲                               │  │
│  │                                                         │ signals                       │  │
│  │                                               ┌─────────┴──────────────────┐            │  │
│  │                                               │  INTUITION ENGINE          │            │  │
│  │                                               │  (subconscious)            │            │  │
│  │                                               │                            │            │  │
│  │                                               │  Async only · Read-only    │            │  │
│  │                                               │  Cannot mutate state       │            │  │
│  │                                               │  Cannot trigger actions    │            │  │
│  │                                               │                            │            │  │
│  │                                               │  Signal types:             │            │  │
│  │                                               │   temporal · repetition    │            │  │
│  │                                               │   consistency · structural │            │  │
│  │                                               │                            │            │  │
│  │                                               │  Ephemeral until promoted  │            │  │
│  │                                               │  by Focus Gate             │            │  │
│  │                                               └────────────────────────────┘            │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ PLANE 2: CONTEXT FIDELITY ────────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌────────────────────────────┐  ┌──────────────────────┐  ┌────────────────────────┐  │  │
│  │  │  ASCC                      │  │  ECS                  │  │  CLT                   │  │  │
│  │  │  Anchored Structured       │  │  Externalized Context │  │  Context Lineage Tree  │  │  │
│  │  │  Context Checkpointing     │  │  Store                │  │                        │  │  │
│  │  │                            │  │                        │  │  Append-only tree of   │  │  │
│  │  │  10 fixed slots per frame: │  │  Content-addressed     │  │  interaction history   │  │  │
│  │  │   intent                   │  │  immutable blobs       │  │                        │  │  │
│  │  │   current_focus            │  │                        │  │  3 node types:         │  │  │
│  │  │   decisions (cap 30)       │  │  7 handle kinds:       │  │   interaction          │  │  │
│  │  │   artifacts (cap 50)       │  │   log · diff · text    │  │   summary              │  │  │
│  │  │   constraints (cap 30)     │  │   json · url           │  │   branch_marker        │  │  │
│  │  │   open_questions (cap 20)  │  │   file_snapshot        │  │                        │  │  │
│  │  │   next_steps (cap 15)      │  │   other                │  │  Compaction inserts —  │  │  │
│  │  │   recent_results (cap 10)  │  │                        │  │  never deletes         │  │  │
│  │  │   failures (cap 20)        │  │  Prompt form:          │  │                        │  │  │
│  │  │   notes (cap 20)           │  │  [HANDLE:kind:id       │  │  Focus State refs      │  │  │
│  │  │                            │  │   "label"]             │  │  exactly 1 CLT head    │  │  │
│  │  │  Delta-only updates        │  │                        │  │                        │  │  │
│  │  │  Deterministic merge rules │  │  Externalize at:       │  │  7 design rules        │  │  │
│  │  │  Section pinning           │  │   >8KB or >800 tokens  │  │  (non-negotiable)      │  │  │
│  │  │  Replaces chat history     │  │  Explicit rehydration  │  │                        │  │  │
│  │  └────────────────────────────┘  └──────────────────────┘  └────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ PLANE 3: MEMORY ──────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌──────────────────────────────────┐    ┌──────────────────────────────────────────┐   │  │
│  │  │  SEMANTIC MEMORY                 │    │  PROCEDURAL MEMORY                       │   │  │
│  │  │                                  │    │                                          │   │  │
│  │  │  Keyed facts/preferences         │    │  Reinforced rules/habits                 │   │  │
│  │  │  {key, value, confidence, TTL}   │    │  {id, rule, weight, scope, enabled}      │   │  │
│  │  │                                  │    │                                          │   │  │
│  │  │  Whitelisted keys → prompt       │    │  Weight decays ×0.99 per tick            │   │  │
│  │  │  Opt-in only · Never inferred    │    │  Scoped: global | frame | project        │   │  │
│  │  └──────────────────────────────────┘    │  Max 5 rules injected per turn           │   │  │
│  │                                           └──────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ PLANE 4: BACKGROUND COGNITION ────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌──────────────────────────────────────────────────────────────────────────────────┐   │  │
│  │  │  WORKER QUEUE                                                                    │   │  │
│  │  │                                                                                  │   │  │
│  │  │  Async · Bounded (100 jobs) · Max 200ms/job · Never blocks hot path              │   │  │
│  │  │                                                                                  │   │  │
│  │  │  ┌────────────────┐ ┌──────────────────┐ ┌───────────────────┐ ┌──────────────┐ │   │  │
│  │  │  │ classify_turn  │ │ extract_ascc_    │ │ detect_repetition │ │ scan_for_    │ │   │  │
│  │  │  │                │ │ delta            │ │                   │ │ errors       │ │   │  │
│  │  │  │ Tags: files,   │ │                  │ │ Repetition hints  │ │              │ │   │  │
│  │  │  │ errors, tools, │ │ Structured delta │ │ → Focus Gate      │ │ Error sigs   │ │   │  │
│  │  │  │ intent shifts  │ │ → Reducer merges │ │                   │ │ w/ severity  │ │   │  │
│  │  │  └────────────────┘ └──────────────────┘ └───────────────────┘ └──────────────┘ │   │  │
│  │  │  ┌────────────────┐                                                              │   │  │
│  │  │  │ suggest_memory │  Workers return RESULTS — never mutate state directly         │   │  │
│  │  │  │                │  Reducer decides whether to accept                            │   │  │
│  │  │  │ Advisory only  │                                                              │   │  │
│  │  │  └────────────────┘                                                              │   │  │
│  │  └──────────────────────────────────────────────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ GOVERNANCE LAYER ─────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌────────────────────────┐  ┌──────────────────────┐  ┌───────────────────────────┐   │  │
│  │  │  UXP                   │  │  UFI                  │  │  AUTONOMY CALIBRATION     │   │  │
│  │  │  User Experience       │  │  User Friction Index  │  │                           │   │  │
│  │  │  Profile               │  │                       │  │  6 dimensions:            │   │  │
│  │  │                        │  │  14 signal types      │  │   Correctness · Stability │   │  │
│  │  │  7 dimensions:         │  │  3 weight tiers:      │  │   Efficiency · Trust      │   │  │
│  │  │   autonomy_tolerance   │  │   High (objective)    │  │   Grounding · Recovery    │   │  │
│  │  │   verbosity_preference │  │   Medium              │  │                           │   │  │
│  │  │   interruption_sensit. │  │   Low (language-only) │  │  ARI: 0–100              │   │  │
│  │  │   explanation_depth    │  │                       │  │  6 levels: AL0→AL5        │   │  │
│  │  │   confirmation_pref.   │  │  Language signals     │  │                           │   │  │
│  │  │   risk_tolerance       │  │  NEVER dominate       │  │  Never self-escalates     │   │  │
│  │  │   review_cadence       │  │  aggregate            │  │  Human grant required     │   │  │
│  │  │                        │  │                       │  │  Scope + TTL mandatory    │   │  │
│  │  │  α ≤ 0.1, window ≥ 30 │  │  UFI ──→ UXP bridge:  │  │                           │   │  │
│  │  │  User override freezes │  │  trend-only learning  │  │  ARI weights:             │   │  │
│  │  │  learning              │  │                       │  │   Outcome 50%             │   │  │
│  │  └────────────────────────┘  └──────────────────────┘  │   Efficiency 20%          │   │  │
│  │                                                         │   Discipline 15%          │   │  │
│  │  ┌────────────────────────┐  ┌──────────────────────┐  │   Safety 15%              │   │  │
│  │  │  AGENT CONSTITUTION    │  │  CONSTITUTION         │  └───────────────────────────┘   │  │
│  │  │  (ACP)                 │  │  SYNTHESIZER (CS)     │                                  │  │
│  │  │                        │  │                       │  ┌───────────────────────────┐   │  │
│  │  │  Versioned, immutable  │  │  Offline, read-only   │  │  RELIABILITY FOCUS MODE   │   │  │
│  │  │  reasoning charter     │  │  analysis assistant   │  │  (RFM)                    │   │  │
│  │  │                        │  │                       │  │                           │   │  │
│  │  │  • Principles          │  │  5-step process:      │  │  R0: Normal               │   │  │
│  │  │  • Self-eval heuristics│  │   1. Evidence aggr.   │  │  R1: Validation            │   │  │
│  │  │  • Autonomy posture    │  │   2. Tension detect   │  │  R2: Regeneration          │   │  │
│  │  │  • Safety rules        │  │   3. Principle map    │  │  R3: Ensemble              │   │  │
│  │  │  • Expression rules    │  │   4. Candidate edit   │  │                           │   │  │
│  │  │                        │  │   5. Draft assembly   │  │  Microcell validators:     │   │  │
│  │  │  SemVer · 1 active     │  │                       │  │   Schema · Constraint     │   │  │
│  │  │  per agent · Rollback  │  │  Human activation     │  │   Consistency · Reference │   │  │
│  │  │                        │  │  required · Min 50    │  │                           │   │  │
│  │  │  Never self-modifies   │  │  tasks evidence       │  │  AIS: ≥0.90 safe         │   │  │
│  │  │  New sessions only     │  │  Never auto-applies   │  │       <0.70 triggers RFM  │   │  │
│  │  └────────────────────────┘  └──────────────────────┘  └───────────────────────────┘   │  │
│  │                                                                                         │  │
│  │  ┌──────────────────────────────────────────────────────────────────────────────────┐   │  │
│  │  │  PROPOSAL RESOLUTION ENGINE (PRE)                                                │   │  │
│  │  │                                                                                  │   │  │
│  │  │  Observations (CLT, refs, telemetry) → always concurrent, append-only            │   │  │
│  │  │  Decisions (focus, thesis, autonomy) → proposals → scored → resolved             │   │  │
│  │  │  Time-bounded windows: 500ms–2000ms · Deterministic scoring · No locks           │   │  │
│  │  └──────────────────────────────────────────────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ THREADS & CONCURRENCY ────────────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌────────────────────┐    ┌──────────────────┐    ┌────────────────────────────────┐  │  │
│  │  │  THREAD             │    │  INSTANCE         │    │  SESSION                       │  │  │
│  │  │  (persistent        │    │  (where)          │    │  (when)                        │  │  │
│  │  │   cognitive         │    │                    │    │                                │  │  │
│  │  │   workspace)        │    │  5 kinds:          │    │  Binds instance to temporal    │  │  │
│  │  │                     │    │   acp | cli | tui  │    │  execution window              │  │  │
│  │  │  Binds:             │    │   gui | background │    │                                │  │  │
│  │  │   Thread Thesis     │    │                    │    │  ATTACHMENT (what):            │  │  │
│  │  │   CLT               │    │  One per harness   │    │   Binds session to thread     │  │  │
│  │  │   Focus Stack       │    │  connection        │    │   Roles: active | assistant   │  │  │
│  │  │   Reference Store   │    │                    │    │          observer | background │  │  │
│  │  │   Telemetry         │    │  Capability-scoped │    │                                │  │  │
│  │  │   Autonomy history  │    │                    │    │  Multiple instances can        │  │  │
│  │  │                     │    │                    │    │  attach to same thread         │  │  │
│  │  │  Ops: Create Resume │    │                    │    │  via PRE resolution            │  │  │
│  │  │  Save Fork Archive  │    │                    │    │                                │  │  │
│  │  │                     │    │                    │    │                                │  │  │
│  │  │  Never share        │    │                    │    │                                │  │  │
│  │  │  mutable state      │    │                    │    │                                │  │  │
│  │  └────────────────────┘    └──────────────────┘    └────────────────────────────────┘  │  │
│  │                                                                                         │  │
│  │  ┌──────────────────────────────────────────────────────────────────────────────────┐   │  │
│  │  │  THREAD THESIS — living semantic anchor per thread                               │   │  │
│  │  │  {primary_intent, secondary_goals, constraints, open_questions,                  │   │  │
│  │  │   assumptions, confidence{score,rationale}, scope{domain,horizon,risk}, sources}  │   │  │
│  │  │  Updated by events, not per-turn · Min confidence delta · Cooldown between       │   │  │
│  │  └──────────────────────────────────────────────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ EXPRESSION ENGINE (HOT PATH) ──── <20ms overhead ────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  Assembles prompt from Focus State — deterministic, bounded, structured                 │  │
│  │                                                                                         │  │
│  │  7 slots (in order):                                                                    │  │
│  │   ┌──────────────┐ ┌───────────────────┐ ┌──────────────────────────────────────────┐  │  │
│  │   │ 1. System    │ │ 2. Operating Rules│ │ 3. Active Focus Frame (ASCC checkpoint)  │  │  │
│  │   │    Header    │ │    (≤5 rules by   │ │    (all 10 slots from active frame)      │  │  │
│  │   │              │ │     weight)       │ │                                          │  │  │
│  │   └──────────────┘ └───────────────────┘ └──────────────────────────────────────────┘  │  │
│  │   ┌──────────────────────┐ ┌───────────────────┐ ┌─────────┐ ┌────────────────────┐   │  │
│  │   │ 4. Parent Context    │ │ 5. Artifact       │ │ 6. User │ │ 7. Execution       │   │  │
│  │   │    (≤2 ancestors,    │ │    Handles (ECS   │ │    Input│ │    Directive        │   │  │
│  │   │     intent+decisions │ │    refs only)     │ │         │ │                    │   │  │
│  │   │     +constraints)    │ │                   │ │         │ │                    │   │  │
│  │   └──────────────────────┘ └───────────────────┘ └─────────┘ └────────────────────┘   │  │
│  │                                                                                         │  │
│  │  Budget: 6000 prompt + 2000 reserve · Degradation cascade (explicit, logged, never      │  │
│  │  silent): drop parents → drop ASCC slots → digest → truncate input → abort (last resort)│  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ OBSERVABILITY ────────────────────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌─────────────────────────┐ ┌──────────────────────┐ ┌───────────────────────────┐    │  │
│  │  │  CTL (Cognitive         │ │  CACHE PERMISSION     │ │  TRAINING DATASET EXPORT  │    │  │
│  │  │  Telemetry Layer)       │ │  MATRIX               │ │                           │    │  │
│  │  │                         │ │                        │ │  4 families:              │    │  │
│  │  │  Passive · Append-only  │ │  5 classes: C0→C4     │ │   focusa_sft             │    │  │
│  │  │  Local SQLite/DuckDB    │ │  10 invalidation      │ │   focusa_preference      │    │  │
│  │  │                         │ │  triggers              │ │   focusa_contrastive     │    │  │
│  │  │  7 event schemas:       │ │  6 cache bust         │ │   focusa_long_horizon    │    │  │
│  │  │   model.tokens          │ │  categories (A–F)     │ │                           │    │  │
│  │  │   focus.transition      │ │                        │ │  JSONL/Parquet output    │    │  │
│  │  │   lineage.node.created  │ │                        │ │  Full provenance         │    │  │
│  │  │   gate.decision         │ │                        │ │  + UXP/UFI signals       │    │  │
│  │  │   tool.call             │ │                        │ │                           │    │  │
│  │  │   ux.signal             │ │                        │ │  Compatible:             │    │  │
│  │  │   autonomy.update       │ │                        │ │   Unsloth · HuggingFace  │    │  │
│  │  │                         │ │                        │ │   Axolotl · TRL          │    │  │
│  │  │  Key derived metrics:   │ │                        │ │                           │    │  │
│  │  │   tokens_per_task       │ │                        │ │                           │    │  │
│  │  │   context_recovery_cost │ │                        │ │                           │    │  │
│  │  │   compression_regret    │ │                        │ │                           │    │  │
│  │  └─────────────────────────┘ └──────────────────────┘ └───────────────────────────┘    │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ PLANE 5: INTERFACES ─────────────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ┌──────────────┐ ┌─────────────────────────┐ ┌─────────────┐ ┌─────────────────────┐ │  │
│  │  │  CLI          │ │  LOCAL HTTP API          │ │  TUI         │ │  MENUBAR GUI        │ │  │
│  │  │  (Rust)       │ │  (Capabilities API)      │ │  (ratatui)   │ │  (SvelteKit+Tauri)  │ │  │
│  │  │               │ │                           │ │              │ │                     │ │  │
│  │  │  14 domains   │ │  127.0.0.1:<port>/v1     │ │  14 views    │ │  Focus Bubble       │ │  │
│  │  │  --json mode  │ │  13 resource namespaces   │ │  Live SSE    │ │  Thought Clouds     │ │  │
│  │  │  Exit: 0–4    │ │  Bearer token auth        │ │  Read-only   │ │  Intuition Pulses   │ │  │
│  │  │               │ │  SSE event streaming      │ │              │ │  Non-modal           │ │  │
│  │  └──────────────┘ └─────────────────────────┘ └─────────────┘ └─────────────────────┘ │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ CAPABILITY PERMISSIONS ── policy always wins over permission ─────────────────────────┐  │
│  │                                                                                         │  │
│  │  Scopes: <domain>:<action> · 3 classes: Read | Command | Administrative                 │  │
│  │  3 token types: Owner (full) | Agent (scoped, revocable) | Integration (read, expirable)│  │
│  │                                                                                         │  │
│  │  AGENT SKILL BUNDLE: 18 skills (8 cognition + 4 telemetry + 2 explain + 4 proposal)    │  │
│  │  Prohibited: set_focus_state · modify_lineage · write_reference · activate_constitution │  │
│  │              escalate_autonomy · approve_export                                          │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
│  ┌─ PERSISTENCE (local-first) ────────────────────────────────────────────────────────────┐  │
│  │                                                                                         │  │
│  │  ~/.focusa/                                                                             │  │
│  │  ├── state/                                                                             │  │
│  │  │   ├── focus_stack.json       Focus Stack snapshot                                    │  │
│  │  │   ├── focus_gate.json        Candidate list (bounded 200)                            │  │
│  │  │   ├── memory.json            Semantic + procedural records                           │  │
│  │  │   └── sessions.json          Session metadata                                        │  │
│  │  ├── ascc/                                                                              │  │
│  │  │   └── <frame_id>.json        ASCC checkpoint per frame                               │  │
│  │  ├── ecs/                                                                               │  │
│  │  │   ├── objects/               Immutable content-addressed blobs                       │  │
│  │  │   ├── handles/               Metadata per handle                                     │  │
│  │  │   └── index.json             Handle index                                            │  │
│  │  ├── events.jsonl               Append-only event log (bounded)                         │  │
│  │  └── config.toml                Single config + env overrides                           │  │
│  │                                                                                         │  │
│  │  All persistence survives daemon restart · Event log supports deterministic replay       │  │
│  └─────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                              │
└──────────────────────────────────────────────────────────────────────────────────────────────┘

┌─ EXTERNAL AUTHORITY ─────────────────────────────────────────────────────────────────────────┐
│                                                                                               │
│  ┌────────────────────────────┐                                                               │
│  │  BEADS                     │  If work is not in Beads, it does not exist.                  │
│  │  (Task Authority)          │  Every Focus Frame maps to a Beads issue.                     │
│  │                            │  tokens_per_task is the canonical optimization metric.         │
│  └────────────────────────────┘                                                               │
│                                                                                               │
│  ┌────────────────────────────┐                                                               │
│  │  MODEL ENDPOINT            │  Focusa is harness-agnostic and model-agnostic.               │
│  │  (any provider)            │  Anthropic · OpenAI · Google · Local · any OpenAI-compatible   │
│  │                            │  If Focusa fails → passthrough (fail-safe, never blocks)      │
│  └────────────────────────────┘                                                               │
│                                                                                               │
└───────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Data Flow Summary

```
                    ┌──────────┐
    User Prompt ───→│  Adapter  │───→ Turn Start
                    └────┬─────┘
                         │
                         ▼
                ┌────────────────┐
                │   Expression   │──→ Assembled Prompt ──→ Model
                │   Engine       │    (7 slots, bounded)
                └───────┬────────┘
                        │ reads
            ┌───────────┼───────────────────┐
            ▼           ▼                   ▼
     ┌────────────┐ ┌───────┐        ┌───────────┐
     │Focus State │ │Memory │        │ECS Handles│
     │(from ASCC) │ │(≤5    │        │(refs only)│
     └────────────┘ │rules) │        └───────────┘
                    └───────┘

    Model Response ──→ Adapter ──→ Turn Complete
                                        │
                           ┌────────────┼──────────────────────┐
                           ▼            ▼                      ▼
                    ┌─────────────┐ ┌──────────┐        ┌──────────┐
                    │  Workers    │ │  CLT      │        │ Telemetry│
                    │  (async)    │ │  (new     │        │ (CTL     │
                    │             │ │   node)   │        │  events) │
                    └──────┬──────┘ └──────────┘        └──────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       ┌──────────┐ ┌──────────┐ ┌──────────┐
       │  ASCC    │ │  Focus   │ │  Memory  │
       │  Delta   │ │  Gate    │ │  Suggest │
       │  (merge) │ │  Signals │ │  (advise)│
       └────┬─────┘ └────┬─────┘ └──────────┘
            │            │
            └──────┬─────┘
                   ▼
            ┌──────────────┐
            │   REDUCER    │──→ New State ──→ Persist
            │  (canonical) │──→ Events    ──→ Log
            └──────────────┘
```

## Architecture

### 5 Planes

**1. Cognitive Control Plane**
- **Focus Stack (HEC)** — Hierarchical Execution Contexts. Models nested task attention. Exactly one active Focus Frame at any time. Every frame maps to a Beads issue. Frames are entered and exited explicitly, with required completion reasons (`goal_achieved`, `blocked`, `abandoned`, `superseded`, `error`). Parent frames contribute selectively to prompts.
- **Focus Gate** — RAS-inspired pre-conscious salience filter. Ingests signals from adapters, workers, daemon, memory. Produces candidates with surface pressure that increases with persistence, goal alignment, risk, and novelty. Pressure decays at `×0.98` per tick. Surface threshold: `2.2`. Candidates surfaced for review only — never auto-switch focus. Supports pinning (immune to decay), suppression (pressure zeroed, audit trail retained), and time-based signals.

**2. Context Fidelity Plane**
- **ASCC** — Anchored Structured Context Checkpointing. Maintains a persistent structured summary per focus frame with 10 fixed slots: `intent`, `current_focus`, `decisions` (cap 30), `artifacts` (cap 50), `constraints` (cap 30), `open_questions` (cap 20), `next_steps` (cap 15), `recent_results` (cap 10), `failures` (cap 20), `notes` (cap 20). Deterministic merge rules per slot. Delta-only updates using turn anchors. Section pinning immune to prompt degradation. Replaces linear chat history.
- **ECS** — Externalized Context Store. Stores large artifacts (diffs, logs, outputs, file snapshots) as immutable content-addressed blobs. Referenced by handles in prompts: `[HANDLE:<kind>:<id> "<label>"]`. 7 handle kinds: `log`, `diff`, `text`, `json`, `url`, `file_snapshot`, `other`. Externalize threshold: **8KB / 800 tokens**. Explicit rehydration only. Session-scoped. Human-pinnable.
- **CLT** — Context Lineage Tree. Append-only tree of interaction history. 3 node types: `interaction`, `summary`, `branch_marker`. 7 non-negotiable design rules (append-only, immutable nodes, never mutates Focus State, Focus State references exactly one CLT head, compaction inserts — never deletes, abandoned branches never erased, fully inspectable/navigable/replayable). Answers "where have we been" not "what do we believe."

**3. Memory Plane**
- **Semantic Memory** — Small explicit facts/preferences. Keyed records (`user.response_style`, `project.name`, etc.) with confidence and TTL. Whitelisted keys injected into prompt.
- **Procedural Memory** — Reinforced rules/habits. Weight-based (decays at `×0.99` per tick). Scoped: `global`, `frame:<id>`, `project:<name>`. At most 5 rules injected per turn, ordered by weight and scope relevance.
- Memory is **opt-in only**. Workers may suggest, never write. No automatic personality drift. No silent preference learning.

**4. Background Cognition Plane**
- **Workers** — Async task queue (bounded 100 jobs). 5 job kinds: `classify_turn`, `extract_ascc_delta`, `detect_repetition`, `scan_for_errors`, `suggest_memory`. Priority: `Low | Normal | High`. Max execution: **200ms per job**. Workers return results — never mutate state directly. Reducer decides whether to accept.
- **Intuition Engine** — Subconscious pattern detector. Runs async only. Emits signals (temporal, repetition, consistency, structural). Cannot block hot path. Cannot mutate Focus State. Cannot trigger actions. All signals are explainable and ephemeral until promoted by Focus Gate.

**5. Interfaces**
- **CLI** — Primary control surface. `focusa <domain> <action> [flags]`. 14 domains mirroring Capabilities API. Machine-readable output (`--json`). Exit codes: 0 success, 1 invalid, 2 policy violation, 3 unauthorized, 4 internal.
- **Local HTTP API** — `http://127.0.0.1:<port>/v1`. 13 resource namespaces. Bearer token auth. SSE event streaming. Commands via `/v1/commands/submit`. All writes audited.
- **TUI** — Full-screen terminal UI (ratatui). 14 navigable domain views. Live updates via SSE. Read-only by default.
- **Menubar GUI** — SvelteKit + Tauri. Ambient cognitive awareness. Focus Bubble (center), Background Thought Clouds (inactive frames), Intuition Pulses (subconscious ripples). Never modal, never demands attention.

### Core Reducer

All state mutations flow through a single deterministic reducer:

```
reduce(state: FocusaState, event: FocusaEvent) → ReductionResult { new_state, emitted_events }
```

15 canonical event types. 7 global invariants (at most one active frame, every frame maps to Beads, Focus State sections always exist, Intuition cannot mutate focus, Focus Gate is advisory only, artifacts immutable once registered, conversation never mutates cognition). State version incremented on every successful reduction. Deterministic, replayable, crash-safe, testable in isolation.

### Governance Layer

**UXP (User Experience Profile)** — Slow-moving calibration of user preferences. 7 canonical dimensions: `autonomy_tolerance`, `verbosity_preference`, `interruption_sensitivity`, `explanation_depth`, `confirmation_preference`, `risk_tolerance`, `review_cadence`. Each dimension has: value (0–1), confidence, citations, scope (user/agent/model/harness), learning rate (`α ≤ 0.1`, window `≥ 30`), user override.

**UFI (User Friction Index)** — Fast-moving interaction cost measurement. 14 signal types in 3 weight tiers: High (task_reopened, manual_override, immediate_correction, undo_or_revert, explicit_rejection), Medium (rephrase, repeat_request, scope_clarification, forced_simplification), Low/Language-only (negation_language, meta_language, impatience_marker). Language signals may NEVER dominate aggregate.

**UFI → UXP Bridge:**
```
UXP_new = clamp(UXP_old × (1 - α) + mean(UFI_window) × α, 0.0, 1.0)
```

**Autonomy Calibration** — Evidence-based trust scoring. 6 dimensions: Correctness, Stability, Efficiency, Trust, Grounding, Recovery. ARI (Autonomy Reliability Index) 0–100. 6 autonomy levels (AL0 advisory → AL5 long-horizon). Promotion requires explicit human grant + minimum ARI + minimum sample size + defined scope + TTL. Never self-escalates.

**Agent Constitution (ACP)** — Versioned, immutable reasoning charter. Behavioral principles, self-evaluation heuristics, autonomy posture, safety rules, expression constraints. Constitutions never self-modify. One active version per agent. Changes apply only to new sessions. Semantic versioning (`MAJOR.MINOR.PATCH`). One-click rollback.

**Constitution Synthesizer (CS)** — Offline, read-only analysis assistant that proposes ACP revisions. 5-step deterministic process: evidence aggregation → normative tension detection → principle impact mapping → candidate rewrite → draft assembly. Requires explicit human activation. Never auto-applies. Evidence-linked diffs. Minimum 50 tasks for analysis window.

**Reliability Focus Mode (RFM)** — 4 levels: R0 (normal) → R1 (validation) → R2 (regeneration) → R3 (ensemble). Microcell validators: Schema, Constraint, Consistency, Reference-Grounding. Artifact Integrity Score (AIS): `≥0.90` safe, `0.70–0.90` degraded, `<0.70` triggers RFM. Agent cannot earn autonomy while losing artifact integrity.

**Proposal Resolution Engine (PRE)** — Timestamped async concurrency across multiple instances. Observations (CLT nodes, refs, telemetry) always concurrent. Decisions (focus changes, thesis updates, autonomy adjustments) expressed as proposals → scored → resolved in time-bounded windows (default 500ms–2000ms).

### Threads, Instances, Sessions

**Thread** = persistent cognitive workspace binding: Thread Thesis, CLT, Focus Stack, Reference Store namespace, telemetry, autonomy history. Operations: Create, Resume, Save, Rename, Fork, Archive. 5 guarantees: threads never share mutable state, one active per session, CLT nodes belong to one thread, telemetry is thread-scoped, autonomy is thread-specific.

**Thread Thesis** = living semantic anchor: primary intent, secondary goals, explicit/implicit constraints, open questions, assumptions, confidence `{score, rationale}`, scope `{domain, time_horizon, risk_level}`, sources. Updated by events, not per-turn. Minimum confidence delta for change. Cooldown between updates.

**Instance** = where (runtime integration point: `acp | cli | tui | gui | background`). **Session** = when (temporal execution window). **Attachment** = what (binding between instance/session and thread, with role: `active | assistant | observer | background`).

### Observability

**Cognitive Telemetry Layer (CTL)** — Passive, append-only. 7 event type schemas: model.tokens, focus.transition, lineage.node.created, gate.decision, tool.call, ux.signal, autonomy.update. Task-centric derived metrics: tokens_per_task, context_recovery_cost, compression_regret. Storage: local SQLite/DuckDB. Exportable for SFT, DPO, RLHF.

**Cache Permission Matrix** — 5 classes: C0 (immutable, safe) → C4 (forbidden). 10 hard invalidation triggers. 6 intentional cache bust categories (A–F): fresh evidence, authority change, compaction, staleness, salience collapse, provider mismatch.

**Training Dataset Export** — 4 families: `focusa_sft`, `focusa_preference`, `focusa_contrastive`, `focusa_long_horizon`. Full schemas with provenance, lineage, UXP/UFI signals. JSONL/Parquet output. Verified compatibility: Unsloth, HuggingFace datasets, Axolotl, TRL.

### Capability Permissions

Scopes: `<domain>:<action>`. 3 classes: Read (safe, non-destructive), Command (intentional mutation, requires policy validation), Administrative (reserved for local owner). 3 token types: Owner (full), Agent (scoped, revocable), Integration (read-only, expirable). **Policy always wins over permission.**

### Agent Skill Bundle

18 skills in 4 categories: Cognition Inspection (8 read-only), Telemetry & Metrics (4 read-only), Explanation & Traceability (2 read-only), Proposal & Request (4 guarded — never enact change). Explicitly prohibited: `set_focus_state`, `modify_lineage`, `write_reference`, `activate_constitution`, `escalate_autonomy`, `approve_export`. Skills reveal truth. Gates decide action. Autonomy is earned.

## Performance Budgets

| Area | Target |
|------|--------|
| Hot path (proxy overhead) | **< 20ms** typical |
| Focus Gate signal ingest | **< 5ms** typical |
| Worker job execution | **< 200ms** per job |
| Prompt assembly | Deterministic, bounded |
| Background tasks | Async, never block hot path |
| ACP proxy overhead | **p50 < 5ms, p95 < 15ms** |
| Long sessions | Hours/days without reset |

## Technology Stack

| Layer | Technology |
|---|---|
| Core Runtime | Rust |
| IPC / API | Local HTTP (JSON) + SSE |
| CLI | Rust |
| TUI | ratatui |
| GUI | SvelteKit + Tauri |
| State Storage | Local filesystem (JSON + JSONL) + SQLite |
| Task Authority | Beads |

## Design Principles

1. **Focus over autonomy** — The system maintains what you're doing, not decides what to do
2. **Structure over prose** — Meaning lives in typed fields, not natural language summaries
3. **Advisory over controlling** — Focus Gate surfaces candidates, never auto-acts
4. **Determinism over magic** — Same state + same input = same prompt, every time
5. **Human intent always wins** — Override anything, rollback anything, inspect everything
6. **Failure must be visible** — No silent truncation, no hidden degradation, no unexplained drift

## Canonical Invariants

These are architectural invariants. Violating any of them is a system fault:

1. At most one active Focus Frame exists at any time
2. Every Focus Frame maps to a Beads issue
3. Focus State sections always exist (may be empty, never absent)
4. Intuition Engine cannot mutate focus
5. Focus Gate is advisory only
6. Artifacts are immutable once registered
7. Conversation never mutates cognition
8. All state transitions are deterministic and replayable from event log
9. No silent prompt changes, no hidden memory writes
10. Session isolation — no cross-session state leakage

## Repository Structure

```
focusa/
├── README.md                          # This file
├── AGENTS.md                          # Agent behavioral protocol
├── .beads/                            # Task tracking (Beads workspace)
├── docs/                              # Authoritative specifications (67 docs)
│   ├── 00-glossary.md                 # Canonical terminology (LOCKED)
│   ├── 01-architecture-overview.md    # System picture & responsibilities
│   ├── core-reducer.md               # Reducer contract, 15 events, 7 invariants
│   ├── G1-detail-03-runtime-daemon.md # AppState, session identity, persistence
│   ├── G1-detail-05-focus-stack-hec.md # FrameRecord, FocusStackState, operations
│   ├── G1-detail-06-focus-gate.md     # Signal/Candidate models, pressure formula
│   ├── G1-07-ascc.md                  # CheckpointRecord, 10 slots, merge rules
│   ├── G1-detail-08-ecs.md            # HandleRef, HandleKind, threshold policy
│   ├── G1-09-memory.md               # SemanticRecord, RuleRecord, decay
│   ├── G1-10-workers.md              # WorkerJob, queue, job definitions
│   ├── G1-detail-11-prompt-assembly.md # 7 slots, token budget, degradation cascade
│   ├── 14-uxp-ufi-schema.md          # UXP dimensions, UFI signals, learning bridge
│   ├── 15-agent-schema.md            # Agent identity, behavioral defaults, policies
│   ├── 16-agent-constitution.md       # ACP schema, principles, safety rules
│   ├── 16-constitution-synthesizer.md # CS process, output schema, review workflow
│   ├── 17-context-lineage-tree.md     # CLT nodes, design rules, compaction
│   ├── 18-cache-permission-matrix.md  # 5 cache classes, permission matrix
│   ├── 19-intentional-cache-busting.md # 6 bust categories A-F
│   ├── 20-training-dataset-schema.md  # 4 dataset families with full schemas
│   ├── 22-data-contribution.md        # ODCL pipeline, contribution policy
│   ├── 23-capabilities-api.md         # 13 namespaces, command model, SSE
│   ├── 24-capabilities-cli.md         # 14 domains, output modes, exit codes
│   ├── 25-capability-permissions.md   # Scope model, 3 token types, enforcement
│   ├── 26-agent-capability-scope.md   # 3 scope tiers, prohibited actions
│   ├── 27-tui-spec.md                # 14 domain views, layout, key bindings
│   ├── 28-ratatui-component-tree.md   # 50+ widget components, data flow
│   ├── 29-telemetry-spec.md           # CTL design, 5 event classes, invariants
│   ├── 30-telemetry-schema.md         # 7 event payload schemas
│   ├── 33-acp-proxy-spec.md           # ACP observation + proxy modes
│   ├── 34-agent-skills-spec.md        # 18 skills, 4 categories, prohibited skills
│   ├── 35-skill-to-capabilities-mapping.md # Exact skill → API endpoint mapping
│   ├── 36-reliability-focus-mode.md   # RFM levels, microcells, AIS thresholds
│   ├── 37-autonomy-calibration-spec.md # 6 dimensions, scoring, calibration suites
│   ├── 38-thread-thesis-spec.md       # Schema, lifecycle, update triggers
│   ├── 39-thread-lifecycle-spec.md    # 6 operations, 5 guarantees
│   ├── 40-instance-session-attachment-spec.md # Schemas, roles, multiplexing
│   ├── 41-proposal-resolution-engine.md # Proposal schema, resolution algorithm
│   ├── PRD.md                         # Product requirements (MVP + updated vision)
│   └── ... (+ remaining specs, bootstrap prompts, UI specs)
├── crates/                            # Rust implementation
│   ├── focusa-core/                   # All cognition
│   ├── focusa-cli/                    # CLI (thin facade)
│   └── focusa-api/                    # HTTP API (thin facade)
├── apps/
│   └── menubar/                       # SvelteKit + Tauri GUI
└── packages/
    ├── ui-tokens/                     # Design tokens
    ├── api-client/                    # API client library
    └── types/                         # Shared TypeScript types
```

## How It Works (Turn-by-Turn)

### 1. Harness sends a user prompt

The adapter intercepts the prompt. Focusa reads current Focus State, ASCC checkpoint, selected memory, and ECS handles.

### 2. Expression Engine assembles the prompt

7 deterministic slots: System Header → Operating Rules (≤5 procedural rules by weight) → Active Focus Frame (ASCC checkpoint, all 10 slots) → Parent Context (optional, bounded) → Artifact Handles (ECS refs only) → User Input → Execution Directive.

Token budget enforced (default: 6000 prompt, 2000 reserve). If exceeded, degradation cascade: drop lowest-priority parent frames → drop non-essential ASCC slots → truncate rehydrated handles → fail only as last resort. All truncation is explicit, logged, reversible.

### 3. Model responds

Response flows back through adapter. Workers fire asynchronously:
- `classify_turn`: tag file paths, errors, tools, intent shifts
- `extract_ascc_delta`: produce structured delta proposal for merge
- `scan_for_errors`: emit error signals with severity
- `suggest_memory`: propose memory updates (advisory only)

### 4. ASCC merges the delta

Deterministic merge rules applied per slot: intent updated only on explicit change marker, decisions appended/deduped (cap 30), artifacts appended/deduped by `(kind + path + label)` (cap 50), open questions removed when answered, next steps replaced with latest, recent results keep last 10 newest-first, failures appended (cap 20), notes append/decay oldest first (cap 20). Revision incremented. Anchor advanced to current turn.

### 5. Focus Gate updates candidates

Signals from workers ingested. Fingerprint dedupe. Pressure updated with base increments (error: +1.2, warning: +0.7, user_input: +0.6, tool_output: +0.5, repeated_pattern: +0.8, assistant_output: +0.2, manual_pin: +2.0), modified by goal alignment (×1.3/×1.1/×0.8), recency (+0.3 within 5 min), risk (+0.4 for error/warning). Decay: `×0.98` per tick. Candidates surfaced when `pressure ≥ 2.2`.

### 6. Everything persisted

Focus State snapshot, ASCC checkpoint, new CLT node, telemetry events, candidate list — all persisted to local storage. Event log is append-only, supports deterministic replay. State survives daemon restart.

### 7. Next turn

Focus State is re-injected. ASCC carries forward. Artifacts remain in ECS. Memory is stable. Compaction can happen to conversation history without loss — because meaning lives in Focus State, not in conversation.

## How Autonomy Is Earned

1. Agent starts at **AL0** (advisory only)
2. System observes performance across 6 dimensions (Correctness, Stability, Efficiency, Trust, Grounding, Recovery)
3. ARI score computed from weighted categories: Outcome 50%, Efficiency 20%, Discipline 15%, Safety 15%
4. When ARI meets threshold with sufficient sample size, Focusa **recommends** promotion
5. Human explicitly grants autonomy: `focusa autonomy grant --level 2 --scope ./repo --ttl 72h`
6. Constitution Synthesizer periodically proposes ACP refinements based on accumulated evidence
7. Human reviews CS drafts, edits wording, activates or discards
8. Agent identity evolves **deliberately**, with full version history and one-click rollback

## Integration Model

### Supported Harnesses (MVP)

- Letta
- Claude Code
- Codex CLI
- Gemini CLI
- Zed (via ACP)
- Any OpenAI-compatible API

### Integration Modes

**Mode A — Wrap Harness CLI (Primary):** Focusa wraps the harness stdin/stdout. Intercepts all I/O. Zero harness modification required.

**Mode B — HTTP Proxy (Optional):** Focusa runs as HTTP proxy between harness and model endpoint.

**Mode C — ACP Proxy:** Focusa terminates ACP client transport, routes bidirectionally, applies full cognitive governance. Latency budget: p50 < 5ms, p95 < 15ms.

### Fail-Safe

If Focusa fails, the adapter passes through raw requests (passthrough mode), emits failure event, does not block the harness. Focusa is invisible unless inspected.

## Relationship to Beads

Beads is the authoritative task system. Focusa governs focus. Beads governs what work exists.

- Every Focus Frame maps to a Beads issue
- If work is not in Beads, it does not exist
- Task lifecycle events (started, completed, abandoned) drive telemetry
- `tokens_per_task` is the canonical optimization metric

## Relationship to Wirebot

Focusa is the cognitive governance layer inside Wirebot. The [FOCUSA_WIREBOT_INTEGRATION.md](../wirebot-core/docs/FOCUSA_WIREBOT_INTEGRATION.md) (152KB, 58 sections) maps every Focusa subsystem onto the Wirebot Memory Bridge infrastructure:

- Focusa Core Reducer → bridge plugin TypeScript
- Focus Stack → Clawdbot session model
- ASCC → Letta memory blocks + workspace files
- ECS → workspace files indexed by memory-core
- Memory → Mem0 (semantic) + local rules (procedural)
- Telemetry → append-only JSONL + systemd journal
- Agent Constitution → workspace SOUL.md

## Success Criteria

The system is working when:

1. Long sessions remain coherent (hours/days without reset)
2. Compaction does not destroy intent
3. Focus never auto-switches
4. Priorities surface meaningfully without interruption
5. Artifacts are never lost
6. Failures are observable
7. Works with real harnesses as a transparent proxy
8. CLI-only usage is sufficient
9. Agent can explain why its behavior changed
10. Autonomy can run for extended periods with verifiable trust

## Status

🚧 **Architecture Locked — Specifications Complete — Pre-Implementation**

67 specification documents (416KB) fully cover every subsystem. Documentation is sufficient to implement without guesswork.

## One-Sentence Summary

> **Focusa preserves continuity of mind across long AI sessions by separating focus, memory, and expression from fragile conversation history.**

## Final Invariant

> **Meaning lives in Focus State, not in conversation.**

---

## License

Proprietary — Startempire Wire

*Part of the Startempire Wire ecosystem — cognitive governance for Wirebot, the AI business operating partner for founders.*
