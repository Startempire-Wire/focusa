# docs/40-instance-session-attachment-spec.md
## Instances, Sessions, Attachments — AUTHORITATIVE SPEC

This document defines the runtime concurrency layer required to support
**multiplexing engineers** (multiple IDEs, terminals, tmux panes, harnesses)
operating simultaneously against Focusa.

It introduces three first-class entities:
- **Instance** (where)
- **Session** (when)
- **Attachment** (what)

and defines their lifecycle, invariants, and required telemetry.

---

## 1. Definitions

### 1.1 Instance
> A concrete runtime integration point connected to the Focusa daemon.

Examples:
- Zed ACP panel connection
- Claude Code CLI process
- Codex CLI session
- Gemini CLI terminal
- a tmux pane runner
- a background worker (Intuition Engine job runner)

An Instance represents a *place* where cognition is invoked.

### 1.2 Session
> A temporal execution window within an Instance.

A Session begins when an Instance connects and ends when it disconnects or times out.
Sessions are used for:
- telemetry continuity
- identifying tool chains and harness context
- bounding caching decisions

### 1.3 Attachment
> A live binding between an Instance/Session and a Thread.

Attachments define which Thread(s) an Instance is interacting with, and how.

---

## 2. Canonical Entity Schemas

### 2.1 Instance Schema

```json
{
  "instance_id": "uuid",
  "created_at": "timestamp",
  "updated_at": "timestamp",

  "kind": "acp | cli | tui | gui | background",
  "integration": {
    "product": "zed | claude_code | codex | gemini | tmux | other",
    "protocol": "acp | stdio | http | grpc | other",
    "version": "string"
  },

  "host": {
    "machine_id": "string",
    "user_id": "string",
    "cwd": "string",
    "repo_root": "string|null"
  },

  "status": "online | offline | degraded",
  "labels": ["string"],

  "capability_scope": {
    "allowed": ["capability_id"],
    "denied": ["capability_id"]
  }
}
```

Notes:
- `machine_id` is stable per device.
- `repo_root` is advisory and may be null.
- `capability_scope` is enforced in the Capabilities API.

---

### 2.2 Session Schema

```json
{
  "session_id": "uuid",
  "instance_id": "uuid",

  "started_at": "timestamp",
  "ended_at": "timestamp|null",
  "status": "active | ended | timed_out",

  "harness": {
    "name": "claude_code | codex_cli | gemini_cli | zed_acp | other",
    "mode": "proxy | observe",
    "details": { "key": "value" }
  },

  "model_context": {
    "provider": "openai | anthropic | google | local | other",
    "model": "string",
    "temperature": 0.0,
    "max_tokens": 0
  },

  "cache_context": {
    "cache_key": "string|null",
    "policy": "normal | conservative | aggressive"
  }
}
```

Notes:
- Sessions may exist without a model_context if in pure observe mode.
- `cache_context` is for Focusa-internal caching, not provider-specific caching.

---

### 2.3 Attachment Schema

```json
{
  "attachment_id": "uuid",
  "thread_id": "uuid",
  "instance_id": "uuid",
  "session_id": "uuid",

  "attached_at": "timestamp",
  "detached_at": "timestamp|null",

  "status": "attached | detached",

  "role": "active | assistant | observer | background",
  "priority": 0,

  "focus_read": true,
  "proposal_write": true,

  "notes": "string|null"
}
```

Role semantics:
- **active**: primary interactive context for that user surface
- **assistant**: secondary surface (may propose, not canonical)
- **observer**: read + telemetry only (no proposals)
- **background**: Intuition Engine work (validators, retrieval, calibration)

---

## 3. Invariants

1. Instances can have many Sessions over time.
2. Sessions belong to exactly one Instance.
3. Attachments bind a Session/Instance to exactly one Thread.
4. A Session can attach to multiple Threads (rare) but MUST declare:
   - one **primary** attachment (highest priority)
5. A Thread can be attached by many Instances simultaneously.
6. Attachments do not grant mutation authority—only proposal authority.

---

## 4. Lifecycle

### 4.1 Instance Lifecycle
- created at first connect
- updated on reconnect or metadata change
- transitions to offline on disconnect
- never deleted automatically (archivable)

### 4.2 Session Lifecycle
- created on connect
- active until disconnect
- ended explicitly or timed out after inactivity

### 4.3 Attachment Lifecycle
- created when Session binds to Thread
- detached on explicit action or session end
- detaching does not delete history

---

## 5. Multiplexing Scenarios

### 5.1 One engineer, many projects
- multiple Instances each attached to different Threads
- each Thread has its own Thesis, CLT, autonomy trajectory

### 5.2 Many Instances, same Thread
- multiple Attachments to same Thread
- all may emit observational events concurrently
- decisional changes occur via proposals + resolution engine

### 5.3 One Instance switches Threads rapidly
- detach/attach is cheap
- Thread resume/rehydration uses Thesis + CLT head + Focus Stack

---

## 6. Telemetry Requirements

Every event must include:
- thread_id (if applicable)
- instance_id
- session_id
- attachment_id (if applicable)

Minimal required telemetry:
- `instance.connected`, `instance.disconnected`
- `session.started`, `session.ended`, `session.timed_out`
- `thread.attached`, `thread.detached`
- `proposal.submitted`, `proposal.resolved`

---

## 7. UI/TUI Requirements

### Menubar must show:
- Active Instances (by product/protocol)
- Active Sessions (harness + mode)
- Attachments (which threads are open where)
- Contention indicators (pending proposals per thread)

---

## 8. Capabilities & Permissions

Instance capability scope is enforced at request-time:
- Instance may be read-only (observer)
- Session may be observe-only
- Attachment role further restricts proposals

No Instance may directly mutate canonical state.

---

## 9. Summary

Instances and Sessions model **runtime reality**.
Attachments bind runtime reality to **cognitive workspaces** (Threads).
This is the concurrency substrate for Focusa.
