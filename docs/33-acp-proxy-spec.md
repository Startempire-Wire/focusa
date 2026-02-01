# docs/33-acp-proxy-spec.md — ACP Proxy & Observation Integration (AUTHORITATIVE)

This document specifies how Focusa integrates with **Agent Client Protocol (ACP)**
via an **Optional Proxy Mode** and a **Passive Observation Mode**.

Focusa’s goal is to support Zed (and any ACP client) while enabling:
- research-grade telemetry
- full Focusa cognition (Focus State, CLT, Gate, Cache rules)
- cross-harness compatibility (Claude, Gemini, Codex, etc.)

This integration MUST preserve:
- ACP correctness
- low latency (imperceptible overhead)
- local-first operation
- Focusa safety and determinism

---

## 0. Canonical Principles

1. **Protocol fidelity**: Focusa must not break ACP semantics.
2. **Observation-first**: passive recording is supported as a low-risk entry.
3. **Cognition requires mediation**: full Focusa capabilities require proxying.
4. **No editor plugins required**: integration should be usable with Zed out of the box.
5. **Low overhead**: async writes, streaming parsing, bounded buffering.
6. **Everything measurable**: ACP is a primary source for CTL telemetry events.
7. **No silent escalation**: proxy mode is an explicit, user-controlled choice.

---

## 1. Definitions

### ACP
Agent Client Protocol: a standardized JSON-RPC protocol between an ACP client
(editor) and an ACP server (agent).

### Focusa ACP Integration Modes
- **Mode A: Passive Observation**
  - Focusa observes ACP traffic and records telemetry.
  - Focusa does not alter prompts, state, or flow.

- **Mode B: Active Cognitive Proxy**
  - Focusa is a transparent JSON-RPC intermediary between client and agent.
  - Focusa applies Focus Gate, Prompt Assembly, CLT tracking, caching policy, etc.

---

## 2. Integration Goals

### Required (MVP)
- Zed ACP sessions work normally through Focusa (proxy mode)
- Focusa can record ACP sessions (observation mode)
- Telemetry events are generated for:
  - session lifecycle
  - prompts/responses
  - tool calls
  - latency
  - token stats (where available)
  - cache events (Focusa-side)
- Mode selection is explicit and reversible

### Non-goals (MVP)
- rewriting ACP spec
- forcing editors to change settings beyond endpoint selection
- model training or distributed uploads (handled by separate subsystem)

---

## 3. Architecture Overview

### 3.1 Mode A — Passive Observation (Wrapper)

```
Zed ACP Client
   ↔ (ACP JSON-RPC over stdio)
Focusa Observer Wrapper
   ↔ (ACP JSON-RPC over stdio)
Agent (Claude/Gemini/Codex ACP server)
```

Focusa:
- launches the agent binary as a subprocess
- pipes stdin/stdout
- parses JSON-RPC frames
- records telemetry
- forwards bytes unmodified

This provides **telemetry only**.

---

### 3.2 Mode B — Active Cognitive Proxy (Preferred)

```
Zed ACP Client
   ↔ (ACP JSON-RPC)
Focusa ACP Proxy (daemon)
   ↔ (ACP JSON-RPC)
Agent ACP server
```

Focusa:
- terminates ACP client transport
- establishes ACP server transport
- routes JSON-RPC messages bidirectionally
- maps ACP session to Focusa Session
- optionally shapes context (Focus Gate, Prompt Assembly)
- records full CTL telemetry

This provides **full Focusa cognition**.

---

## 4. Mode Selection & UX

### 4.1 User Control
Users enable ACP integration via config:

```json
{
  "acp": {
    "enabled": true,
    "mode": "observe | proxy",
    "listen": "127.0.0.1:4778",
    "target_agent": {
      "kind": "claude | gemini | codex | other",
      "transport": "stdio | tcp | ws",
      "command": ["claude", "--acp"]
    }
  }
}
```

### 4.2 Editor Configuration (Zed)
Zed must be configured to point to Focusa as the ACP server endpoint (proxy mode),
or to run the Focusa wrapper command (observation mode).

Focusa must never require editor-specific plugins.

---

## 5. ACP Session Mapping to Focusa

### 5.1 Session Identity
Each ACP session maps to a Focusa session:

- `acp_session_id` (from ACP lifecycle)
- `focusa_session_id` (Focusa internal UUID)

Mapping is stored in telemetry and session registry.

### 5.2 Agent Identity
ACP agent identity is mapped to Focusa’s Agent Registry:

- `agent_id` (Focusa)
- `harness_type = "acp"`
- `provider/model` if available or inferable

---

## 6. Message Routing & Transformation Rules

### 6.1 Passive Observation Mode
- no transformation is allowed
- the wrapper forwards bytes unmodified
- Focusa records events only

### 6.2 Active Proxy Mode
- Focusa MUST preserve ACP message shape and ordering
- Focusa MAY:
  - add tracing metadata via allowed fields (if ACP supports)
  - enforce timeouts
  - apply Focusa prompt assembly BEFORE forwarding a prompt to the agent

If ACP prohibits payload changes, Focusa must instead:
- keep original prompt intact
- inject Focusa prompt assembly as a preamble segment in the ACP prompt content
- or rely on agent tool-call affordances to fetch Focusa state externally

(Exact strategy is implementation-defined; correctness is mandatory.)

---

## 7. Focusa Cognitive Hooks (Proxy Mode)

### 7.1 Focus Gate
On `session/prompt`:
1. Read current Focus State
2. Evaluate candidate relevance and freshness
3. Decide:
   - accept as-is
   - request additional refs
   - trigger cache bust
   - compact history / rehydrate refs

### 7.2 Prompt Assembly
Focusa assembles:
- constitution excerpts (stable)
- minimal Focus State snapshot
- CLT deltas
- salient references
- tool breadcrumbs
- user prompt (ACP prompt payload)

Output becomes the forwarded ACP prompt content (as allowed).

### 7.3 CLT Updates
On each prompt-response cycle:
- append CLT node: `interaction`
- link to refs created/used
- emit summary nodes upon compaction

### 7.4 Telemetry (CTL)
For every ACP message:
- emit telemetry events:
  - `acp.rpc.in`
  - `acp.rpc.out`
  - plus derived events: `model.tokens`, `tool.call`, etc.

---

## 8. Token & Cost Tracking in ACP

### 8.1 Primary Sources
- If ACP provides token usage in payloads, record directly.
- If provider response includes token counts, record those.
- Otherwise:
  - estimate via tokenizer locally (flag as `estimated=true`)

### 8.2 CTL Events
- `model.tokens` events MUST include:
  - prompt_tokens
  - completion_tokens
  - cached_tokens (if Focusa caching applies)
  - latency_ms
  - model/provider identifiers

---

## 9. Performance Requirements

### 9.1 Latency Budget
Proxy overhead target:
- **p50: < 5ms**
- **p95: < 15ms**

All telemetry writes must be:
- async
- batched
- non-blocking

### 9.2 Backpressure
If telemetry queue is saturated:
- drop low-priority telemetry
- never block ACP routing

High-priority telemetry (errors, session boundaries) must always persist.

---

## 10. Failure Modes & Safety

### 10.1 Agent Crash
- proxy detects EOF / disconnect
- emits `acp.session.error`
- surfaces status to UI/TUI
- allows reconnect / restart

### 10.2 Focusa Crash
- in proxy mode: ACP session ends (expected)
- in observe mode: wrapper ends with agent
- data integrity preserved via WAL

### 10.3 Protocol Mismatch
- detect ACP version mismatch early
- provide clear error and fallback guidance

---

## 11. Security Boundaries

- default bind: localhost only
- require local auth token for API access
- do not write raw prompts to export by default
- apply local redaction policies for logs if enabled

---

## 12. Capabilities API Additions

New domain namespace:

- `acp.*`

Required endpoints:
- `GET /v1/acp/status`
- `GET /v1/acp/sessions`
- `GET /v1/acp/session/{id}`
- `GET /v1/acp/events?session_id=...`

These must be reflected into `telemetry.*` where applicable.

---

## 13. CLI Additions

```bash
focusa acp status
focusa acp sessions
focusa acp inspect <acp_session_id>
focusa acp tail --session <id>
```

---

## 14. TUI Additions

Add navigation item:
- `ACP`

Subviews:
- live sessions
- message stream
- token + latency summary
- errors

ACP view is primarily a window into `telemetry.*` filtered by `harness=acp`.

---

## 15. Implementation Milestones

### Phase 1 — Passive Observation (Wrapper)
- stdio interception
- JSON-RPC framing
- CTL event capture
- no transformations

### Phase 2 — Proxy (Read-only Cognition)
- bidirectional routing via daemon endpoint
- session mapping
- CLT node creation
- telemetry + UI

### Phase 3 — Full Cognitive Proxy
- Focus Gate
- Prompt Assembly
- cache policy enforcement
- autonomy hooks

Each phase is useful independently.

---

## 16. Canonical Rule

> **Observation is optional.  
> Proxying is explicit.  
> Cognition is earned.**
