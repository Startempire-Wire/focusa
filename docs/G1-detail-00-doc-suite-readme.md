# docs/00-README.md — Focusa MVP Documentation Suite

This documentation suite fully specifies the Focusa MVP architecture. It is written to allow an engineer (human or agent) to implement the MVP with minimal ambiguity.

## What Focusa Is (MVP)
Focusa is a **local cognitive runtime** that sits between an existing harness (Letta, Claude Code, Codex, Gemini CLI, etc.) and the model/API. It behaves like a **cognitive proxy**:

- It **does not** replace the harness.
- It **does not** modify the model.
- It **does** govern focus over time, context fidelity, and memory injection.
- It **does** provide a CLI, local API, and minimal menubar UI.

## MVP Promise
- Maintain stable focus across long sessions.
- Prevent context collapse using structured incremental checkpointing.
- Externalize bulky artifacts to avoid wasting tokens.
- Allow priorities to **emerge organically** via Focus Gate (pre-conscious salience filter).
- Be fast/imperceptible.

## Non-Goals (MVP)
- No model training or RL.
- No inference kernel work (no attention hacks).
- No Letta-specific code in core (only thin adapter scaffolding).
- No fully interactive infinite canvas yet.
- No multi-agent coordination.

## Doc Map (Implement in this order)
1. `01-architecture-overview.md` (system picture & responsibilities)
2. `02-repo-layout.md` (monorepo, crates, packages)
3. `03-runtime-daemon.md` (Rust daemon, state model, event loop)
4. `04-proxy-adapter.md` (how Focusa wraps harnesses generically)
5. `05-focus-stack-hec.md` (focus hierarchy)
6. `06-focus-gate.md` (salience and organic priority emergence)
7. `07-ascc.md` (anchored structured context checkpointing)
8. `08-ecs.md` (externalized context store, handles, retrieval)
9. `09-memory.md` (semantic/procedural minimal memory)
10. `10-workers.md` (background worker pipeline)
11. `11-prompt-assembly.md` (prompt budgets, slot structure, delta injection)
12. `12-api.md` (local HTTP/WebSocket API)
13. `13-cli.md` (command contract)
14. `14-gui-menubar.md` (Tauri + SvelteKit menubar MVP)
15. `15-events-observability.md` (event schema, logging, tracing)
16. `16-testing.md` (test plan & acceptance tests)

## Canonical Terms
- **Focusa**: the whole system.
- **Daemon**: the local resident process.
- **Focus Stack / HEC**: hierarchical execution contexts (frames).
- **Focus Gate**: RAS-inspired pre-conscious salience filter.
- **ASCC**: Anchored Structured Context Checkpointing.
- **ECS**: Externalized Context Store.
- **Handle**: a typed reference to externalized artifacts.
- **Prompt Assembly**: deterministic prompt construction from slots + selected context.

## Compatibility Goal
Focusa must be able to work in **proxy mode** with any harness by:
- wrapping a CLI process and intercepting I/O, or
- acting as an HTTP proxy (where applicable),
- without requiring harness modifications.

Initial testing will focus on Letta usage patterns, but core must remain generic.

---

# UPDATE

# docs/00-README.md (UPDATED) — Focusa MVP Documentation Suite

## Addendum: Boundary, Safety, and Trust Guarantees (MVP-Complete)

The Focusa MVP explicitly guarantees the following properties, which are now considered **architectural invariants**:

### Identity & Boundary Guarantees
- Every interaction belongs to a **Session**
- Sessions are isolated
- Focus, memory, and context never leak across sessions unless explicitly merged

### Determinism Guarantees
- Prompt assembly is deterministic given the same state + input
- Reducer state transitions are replayable from events
- No hidden prompt rewriting

### Trust & Control Guarantees
- Focus Gate is advisory only
- No automatic focus switching
- No automatic memory writes
- Human intent always wins

### Failure Transparency
- Prompt degradation is explicit and observable
- Silent truncation is forbidden
- All degradations emit events and user-visible warnings

These guarantees are now normative for all implementations.
