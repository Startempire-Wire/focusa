# docs/09-proxy-adapter.md — Proxy & Harness Adapters (MVP)

## Purpose

Focusa integrates as a **transparent local proxy** between AI harnesses and model backends.

No harness internals are modified.

---

## Adapter Responsibilities

- Intercept model requests
- Invoke Expression Engine
- Inject Focus State
- Forward requests to model
- Capture responses
- Emit events

---

## Supported Harnesses (MVP)

- Letta
- Claude Code
- Codex CLI
- Gemini CLI
- Generic OpenAI-compatible APIs

---

## Adapter Model

Each adapter:
- normalizes requests
- preserves original semantics
- adds no behavioral changes

---

## Failure Handling

If Focusa fails:
- adapter passes through raw request
- emits failure event
- does not block harness

---

## Performance Constraints

- <20ms overhead typical
- Zero blocking
- Async I/O only

---

## Security

- Local-only by default
- No telemetry
- No prompt inspection unless enabled

---

## Acceptance Criteria

- Harness behavior unchanged
- Focusa is invisible unless inspected
- Fail-safe passthrough works

---

## Summary

Adapters make Focusa **ubiquitous without intrusion**.
