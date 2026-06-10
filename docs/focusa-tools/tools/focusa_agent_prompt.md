# `focusa_agent_prompt`

**Family:** `focus_state`  
**Label:** Agent Prompt

## Purpose

Retrieve the in-band reminder that Pi clients should use `focusa_*` tools for daemon interactions instead of raw `curl`/`fetch` calls.

## When to use

Use `focusa_agent_prompt` when:

- Starting a Pi session to re-establish the canonical tool-call path.
- Reviewing whether the current interaction is in Pi-aware mode.
- Diagnosing why non-canonical API calls are still happening.

## When not to use

Do not rely on this for normal project planning, trajectory, or workpoint state changes; use the specialized tools for those operations.

## Example usage

```text
focusa tool focusa_agent_prompt
```

## Expected result

A structured response with the canonical reminder payload (for Pi clients, with response header `x-focusa-agent-prompt: focusa_*`) including:

- `is_agent`
- `preferred_layer`
- `rule`
- `tool_families`
- `tool_count`
- `next_tools`

## Recovery notes

- If this call is unavailable, check `focusa_tool_doctor` and verify `/v1/health`.
- Non-Pi traffic may return a non-agent minimal response (`is_agent: false`).
- If reminder is still missing on Pi traffic, confirm headers:
  - `X-Focusa-Client: pi`
  - `X-Extension-Token: focusa-pi-*`

## Contract summary

- Family: FocusState.
- Side effects: `read_only`.
- Result envelope: `tool_result_v1`.
- API route: `GET /v1/agent/prompt`.
- CLI: none (Pi-only).
- Parity: `pi_only`; exemptions: `pi_only`, `domain_cli_only`.
- Core surface: Pi runtime reminder and tool-discovery surface.
- Live check: contract_static plus `/v1/agent/prompt` with Pi headers.
- Contract source: `apps/pi-extension/src/tool-contracts.ts`.