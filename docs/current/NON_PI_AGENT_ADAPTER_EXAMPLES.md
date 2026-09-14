# Non-Pi Agent Adapter Examples

These examples show how non-Pi agents satisfy `AGENT_ADAPTER_CONTRACT.md` while staying thin. The Focusa daemon/core remains cognitive authority; adapters only call CLI/HTTP/MCP surfaces and display `tool_result_v1` envelopes.

## Universal startup packet

All examples start with the same authority-safe sequence:

```bash
export FOCUSA_PROJECT_ROOT=${FOCUSA_PROJECT_ROOT:-$PWD}
export FOCUSA_CONTINUITY_ID=${FOCUSA_CONTINUITY_ID:-focusa-cont-root-20b6704c-5a49-4d9d-a4b6-a30bf45bfc61}
focusa awareness card --json
focusa project verify --project-root "$FOCUSA_PROJECT_ROOT" --json
focusa workpoint resume --project-root "$FOCUSA_PROJECT_ROOT" --continuity-id "$FOCUSA_CONTINUITY_ID" --json
focusa context-cognition render --project-root "$FOCUSA_PROJECT_ROOT" --continuity-id "$FOCUSA_CONTINUITY_ID"
```

## Codex CLI

```bash
codex --dangerously-bypass-approvals-and-sandbox=false \
  "Read docs/current/AGENT_ADAPTER_CONTRACT.md, run the universal startup packet, then continue only from canonical Workpoint scope."
```

Before mutation, Codex must run:

```bash
focusa action preflight --current-ask "$CURRENT_ASK" --target "$TARGET" --kind code_edit --project-root "$FOCUSA_PROJECT_ROOT" --json
```

## Claude Code

```bash
claude "Use Focusa as external authority: run awareness card, project verify, workpoint resume, and context-cognition render before planning. Capture evidence after checks."
```

Evidence handoff:

```bash
focusa evidence capture --target-ref "$TARGET" --result "$RESULT" --evidence-ref "$EVIDENCE_REF" --project-root "$FOCUSA_PROJECT_ROOT" --json
```

## OpenCode

```bash
opencode "Load docs/current/NON_PI_AGENT_ADAPTER_EXAMPLES.md. Use Focusa CLI/HTTP as authority and never infer canonical state from transcript tail."
```

OpenCode MCP mode may call the same routes through an MCP bridge instead of shell commands.

## OpenClaw / Wirebot

```bash
openclaw "Fetch Focusa awareness card and Workpoint resume packet. Treat project_root + continuity_id as scope authority. Render tool_result_v1 status in replies."
```

OpenClaw/Wirebot should show the compact Utility Card plus current Workpoint next action before taking durable action.

## Generic shell agent

```bash
curl -fsS http://127.0.0.1:8787/v1/awareness/card | jq .
curl -fsS -X POST http://127.0.0.1:8787/v1/project/verify \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$FOCUSA_PROJECT_ROOT\"}" | jq .
curl -fsS -X POST http://127.0.0.1:8787/v1/workpoint/resume \
  -H 'content-type: application/json' \
  -d "{\"project_root\":\"$FOCUSA_PROJECT_ROOT\",\"continuity_id\":\"$FOCUSA_CONTINUITY_ID\"}" | jq .
```

## MCP-compatible agents

Expose Focusa CLI/HTTP routes as MCP tools with these tool names or equivalents:

- `focusa_awareness_card`
- `focusa_project_verify`
- `focusa_workpoint_resume`
- `focusa_workpoint_checkpoint`
- `focusa_evidence_capture`
- `focusa_workpoint_link_evidence`
- `focusa_action_preflight`
- `focusa_context_cognition_render`

MCP wrappers must pass through `canonical`, `advisory`, `degraded`, `failure_class`, `retry`, `next_tools`, and `evidence_refs` fields without rewriting authority semantics.

## Grant-scoped SMS OTP injection

The grant and target handles below are opaque values issued by the credential authority/control plane. Never place an OTP value in an argument, response, log, screenshot, or model prompt.

CLI:

```bash
focusa sms otp-challenge --provider github.com --target-handle "$TARGET_HANDLE" \
  --grant-id "$GRANT_ID" --consumer-ref "$CONSUMER_REF" --json
focusa sms otp-inject --challenge-handle "$CHALLENGE_HANDLE" --target-handle "$TARGET_HANDLE" \
  --grant-id "$GRANT_ID" --consumer-ref "$CONSUMER_REF" --json
```

REST adapters send the same fields to `POST /v1/sms/otp/challenges` and `POST /v1/sms/otp/inject`, preserving `tool_result_v1`. MCP/OpenClaw adapters use `focusa_sms_otp_challenge` then `focusa_sms_otp_inject`. They may display challenge/status handles and `injected=true`; they must not add a reveal step.

Thread/read/search/send/event operations require separately issued capabilities even when the same consumer holds an OTP grant. Send additionally requires `confirm=true` and an idempotency key. Revoke additionally requires explicit owner confirmation.

## Required contract checklist

- Read awareness card.
- Verify project identity.
- Resume Workpoint.
- Create Workpoint checkpoint.
- Capture evidence.
- Link evidence.
- Run Context Authority preflight.
- Render Context Cognition compact packet.
- Surface `tool_result_v1` envelopes.
- Respect canonical/advisory/degraded states.
