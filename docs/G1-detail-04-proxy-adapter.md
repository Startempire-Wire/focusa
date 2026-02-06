# docs/04-proxy-adapter.md — Proxy Mode & Harness-Agnostic Integration

## Goal
Focusa must work with **any harness** by behaving like a proxy layer that:
- intercepts prompts and tool outputs (where observable),
- injects structured context and references,
- returns model/harness responses transparently.

MVP focuses on generic I/O methods:
1) CLI subprocess wrapping (primary)
2) HTTP proxying (optional)

No harness internals required.

## Integration Modes

### Mode A — Wrap Harness CLI (MVP Primary)
Focusa adapter starts a harness CLI process and mediates I/O.

#### "Magic" UX (Desired)
Users should be able to run harness CLIs directly (e.g. `pi`, `claude`) and still have Focusa capture turns.

Recommended implementation: PATH shims that transparently exec:
- `focusa wrap -- <harness> <args...>`

See: `docs/42-magic-harness-shims.md`

Example:
- Harness: `letta run ...` (or equivalent)
- Focusa wrapper:
  - reads user prompt
  - calls Focusa daemon to assemble prompt
  - sends to harness stdin
  - streams harness stdout back
  - sends transcript + artifacts back to daemon

#### Requirements
- Works with line-delimited and streaming output.
- Handles cancellation (Ctrl-C) gracefully.
- Captures tool outputs if harness prints them (best effort).

### Mode B — HTTP Proxy (Optional)
If a harness uses HTTP calls to an LLM provider, Focusa can:
- accept the same request schema,
- rewrite prompt/messages,
- forward upstream,
- return response.

MVP can omit if CLI wrapping is sufficient.

## Adapter Responsibilities (MVP)
- Create a `Turn` boundary:
  - each prompt/response pair is one Turn
- Provide `correlation_id` per Turn to daemon
- Emit `Signal`s to Focus Gate:
  - user input received
  - tool output captured
  - errors
  - repeated warnings
- Provide transcript to ASCC:
  - minimal: user prompt + assistant response (redacted if needed)
- Externalize blobs:
  - if tool output exceeds thresholds, send to ECS and replace with handle

## Adapter Contract with Daemon

### Required Daemon Endpoints
- `POST /v1/turn/start`
- `POST /v1/turn/append` (streaming chunks optional)
- `POST /v1/turn/complete`
- `POST /v1/prompt/assemble`

### Turn Data Shapes
TurnStart:
- `turn_id`
- `adapter_id`
- `harness_name`
- `timestamp`

PromptAssembleRequest:
- `turn_id`
- `raw_user_input`
- `harness_context` (optional, e.g., system prompt template)
- `max_tokens_budget` (optional)

PromptAssembleResponse:
- `assembled_prompt` (string or messages array)
- `handles_used[]`
- `context_stats` (token estimates)

TurnComplete:
- `turn_id`
- `assistant_output`
- `artifacts[]` (optional; may be handles)
- `errors[]` (optional)

## Prompt Shape Normalization
Focusa core must support two outbound formats:
1) Plain string prompt
2) Chat messages array:
   - `{role: "system"|"user"|"assistant", content: "..."}`
Adapter chooses based on harness needs.

## Thresholds (MVP Defaults)
- externalize any single blob > 8KB OR estimated > 800 tokens
- keep assembled prompt under a configurable budget (e.g., 6k tokens)

## Security/Privacy (MVP)
- Focusa stores locally only.
- If transcripts are sensitive, allow a config:
  - store only structured summary (ASCC) not full transcript
  - or store transcript but redacted

## Validation Checklist (MVP)
- Wrapper can run a harness command unchanged.
- Responses match baseline behavior (no corruption).
- Overhead is minimal.
- Prompt length stabilizes over time due to ASCC+ECS.

---

# UPDATE

# docs/04-proxy-adapter.md (UPDATED) — Adapter Capabilities

## Adapter Capability Declaration (Added)

Each adapter MUST declare:

```json
{
  "streaming": true,
  "tool_output_capture": false,
  "structured_messages": true
}
```

### Usage
Focusa uses this to:
- disable unsupported features
- avoid false assumptions
- adjust ASCC expectations

---

## Invariants
- Adapter limitations never degrade correctness
- Missing signals reduce capability, not safety
