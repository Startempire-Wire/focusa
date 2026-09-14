# Agent Adapter Contract

Focusa adapters are thin integration layers. The Focusa daemon/core remains the cognitive authority for Workpoints, Trajectory, Evidence, Context Authority, Context Cognition, and tool envelopes.

## Non-negotiable rule

Adapters stay thin. Focusa daemon/core remains cognitive authority. Adapters must not invent canonical state, merge workstreams, or promote advisory packets to authority.

## Minimal adapter contract

Every agent adapter supports these capabilities, either through Pi tools, Focusa CLI, direct HTTP, or an MCP/tool bridge:

1. **Read awareness card** — `GET /v1/awareness/card` or `focusa awareness card`.
2. **Verify project identity** — `GET /v1/project/verify` / `focusa project verify` / equivalent project identity route.
3. **Resume Workpoint** — `/v1/workpoint/resume` / `focusa workpoint resume`.
4. **Create Workpoint checkpoint** — `/v1/workpoint/checkpoint` / `focusa workpoint checkpoint`.
5. **Capture evidence** — `/v1/evidence/capture` / `focusa_evidence_capture`.
6. **Link evidence** — `/v1/workpoint/link-evidence` / `focusa_workpoint_link_evidence`.
7. **Run Context Authority preflight** — `focusa action preflight` or `/v1/action/preflight` before risky mutation.
8. **Render Context Cognition compact packet** — `/v1/context-cognition/render` or `focusa context-cognition render`.
9. **Surface `tool_result_v1` envelopes** — expose `status`, `failure_class`, `canonical`, `advisory`, `degraded`, `next_tools`, evidence refs, and recovery hints.
10. **Respect canonical/advisory/degraded states** — canonical state requires verified scope; advisory/degraded packets never override operator steering or Workpoint authority.

## Target adapter classes

- Pi
- Codex CLI
- Claude Code
- OpenCode
- OpenClaw
- generic shell agent
- MCP-compatible agents

## Authority boundaries

- Operator steering wins.
- `project_root + continuity_id` is the authority boundary for project/workstream state.
- `session_id` is temporal metadata, not authority.
- Context Cognition, Project Card, Prediction, Metacognition, and Call Stack Design are advisory unless explicitly linked through Workpoint/Trajectory/Evidence paths.
- Transcript tail is never authority after compaction or tool-output flood.

## Risky mutation preflight

Before risky mutation, every adapter must classify prompt mode, inspect environment contract/runtime inventory, run Context Authority preflight, and proceed only when verdict allows.

Risky mutation includes deploy, daemon restart, binary replacement, git push, destructive file operation, database migration, generated-code overwrite, secret/config change, release publish, broad refactor, cross-project edit, and pairing/install/update ambiguity.

## Communications capability contract

Communications adapters remain thin and connector-neutral. They pass opaque handles and `focusa.tool_result_v1`; they never receive connector profile state, pairing payloads, provider cookies, or OTP values.

- Health/enrollment are value-free diagnostics.
- Every list/read/search/send/event/checkpoint/revoke/challenge/inject request carries an active `grant_id` and attributable `consumer_ref` except value-free health/enrollment.
- OTP challenge registration binds provider + exact target before delivery. Injection requires the same grant, consumer, challenge, and target; successful output is `injected=true`, never the value.
- `inject_otp` does not imply thread/read/search/send/event authority. Each broader operation requires its own capability.
- Mutations retain confirmation/idempotency rules; revoke requires explicit owner confirmation and cryptographic erasure.
- Adapters must reject noncanonical broker envelopes or responses containing credential/OTP/pairing fields.

Canonical generated schemas: `docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json` and `docs/focusa-tools/tools/focusa_sms_*.md`.

## Failure behavior

Adapters must:

- show `failure_class` and recovery hints instead of hiding daemon errors
- preserve proof handles rather than raw logs
- avoid leaking secrets/tokens/private file contents into public cards
- checkpoint before compaction/model switch/risky continuation
- recover from stale/degraded state by project identity → trajectory view → Workpoint resume/checkpoint → evidence capture

## Verification

Static guard: `tests/agent_adapter_contract_static_test.sh`.

Related docs:

- `docs/current/NON_PI_AGENT_FOCUSA_USAGE.md`
- `docs/current/AUTHORITY_MODEL.md`
- `docs/current/GOLDEN_WORKFLOW.md`
- `docs/current/CONTEXT_AUTHORITY_CURRENT.md`
- `docs/current/TOOL_RESULT_ENVELOPE_V1.md`
