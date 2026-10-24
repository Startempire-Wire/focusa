# Focusa Pi Extension

Focusa Pi Bridge is the Pi coding-agent extension that registers Focusa tools, lifecycle hooks, compaction/session handling, and Focusa skills used by Pi sessions.

## Source surface and installed activation

Use [the shared project journey](../../docs/agent/02-focusa-cohesive-project-flow.md). Discover the current process's tool count and registry digest rather than assume a historical inventory. Source registration and generated projections do not prove the installed package or another harness loaded them. Startup/reload now renders the Focusa Utility Card; recovery composes `focusa_tool_doctor`, project identity, Trajectory, Workpoint checkpoint/resume, evidence capture/linking, prediction evaluation, metacognition, and work-loop routes. Context Cognition, Project Card/Genesis, Temporal Authority, preload, Silent Sessions, browser/UIAI workflows, device pairing, and Tool Discovery remain available through the same scope and operator-steering boundary.

Project background dispatch must carry the current invocation's verified attachment in a private child environment, not infer it from cwd or inherit a process-global startup identity. Both single and batch tools fail closed before spawning when that context is missing or mismatched; daemon admission and existing receipts remain controlling. See [Spec188 and its dependency graph](../../docs/188-focusa-invocation-bound-background-dispatch-repair-spec.md). Source implementation alone does not establish installed activation or issue closure.

## Project foundation and control observations

Before dependent operations, follow the shared project journey linked above: read the relevant guidance, verify the intended project, and reuse or complete its existing operator-confirmed trajectory and linked action. A candidate goal or invocation metadata is not execution admission; missing setup must be identified rather than described as a global work stop. The North Star diagnostic reads the current verified session scope; `project_root` and `continuity_id` are not selector inputs for that tool.

Writer status reports `none` only for an explicit null in a supported successful response; failed, missing or unsupported observations are `unverified`. An unverified lease is not evidence of another writer. Control preflight is a route preview only: it sends no control request and does not check ownership, budgets or execution admission. Unsent requests retain that distinction; daemon refusals and fencing remain unchanged. Other existing authorizations are not revoked by these observations.

The opt-in `tests/693-work-loop-live-reporting.test.mjs` exercises the actual native SDK and explicitly selected canonical daemon scope using reads only by default. The separate `FOCUSA_LIVE_PAUSE_RECOVERY=1` mode permits one already-authorized pause only after observing an idle, unclaimed scope with zero remaining wall time; it checks unchanged budget bounds and writer state. The flag grants no authority and bypasses no daemon checks. Neither mode starts/resumes workers, renews budgets or measures model compliance. State readback alone is not durability proof; source verification is not installed activation.

## North Star diagnostics

A saved trajectory can remain current while the saved action needs reconciliation
with a new instruction. The card leads with the condition causing its blocked or
stale status; an informational missing gap must not hide a request mismatch or a
missing Workpoint. Operator steering is not described as an expired goal or a
missing saved checkpoint. This presentation does not change authority decisions;
resuming and verifying the current action remains necessary when it is stale.

Startup and explicit resume share request-binding checks. Only an affirmative,
canonical reply for the unchanged ask, turn, session, project and continuity may
be adopted. A late reply cannot replace newer saved work. An explicit alternate
`current_ask` is advisory for the active session, and legacy replies lacking
positive authority flags require supported reconciliation rather than assumed
permission. Successful adoption binds the evaluated ask, not newer input.

CurrentAsk retains the complete cleaned instruction; only the operator-facing
label is bounded. The hash-verified recovery sidecar stores exact request text
with `textIsPreview: false`. Recovery requires the anchor's exact digest and
revision plus the matching project/session; a corrupt or missing current
artifact cannot silently substitute an older or newer request. The bounded
native anchor carries a preview marked `textIsPreview: true`, never a substitute
for the exact sidecar. Restoration
checks session, project and continuity and accepts only explicitly complete
request text. Missing sidecars and legacy snapshots of unknown completeness
remain recovery context; frame goals cannot reconstruct request authority.
Existing sidecar and native-anchor size limits remain enforced. Original input
must be recovered before treating incomplete saved text as a current request.

The resumed-context projection reserves required request, scope warning,
identity, goal/action, blocker, restriction, evidence/receipt and rehydration
meaning before optional temporal detail. A supplied temporal effect-guard
reference also remains required. Required fields are never individually clipped;
if they cannot fit the selected 900/600/400/250 budget estimate, rendering
explicitly blocks rather than dropping a late restriction. Unverified request
completeness retains advisory review context but blocks saved-action resumption.
Provider/tokenizer/model-delivery qualification remains separate from this
source-level budget estimate. The normal producer/caller lifecycle binds the
complete current input and actual source turn to a durable prepared artifact,
checks current request/scope after each asynchronous response, and rejects
legacy partial or mismatched handoffs. Preparation uses the actual reducer
version and existing writer/SQLite fences; committed local deltas are reconciled
without reapplication. Pending deltas block rather than becoming count-as-commit
claims. Identical epoch retries reuse the artifact; changed epoch input conflicts.
Prepared-artifact verification is not model-consumption evidence. See
`docs/current/COMPACTION_FALLBACKS.md` for the exact compatibility/failure posture.

### Exact scoped preload artifacts (opt-in protocol 2)

Preload build/render/verify/doctor and delivery tools accept `artifact_version: 2`
with the exact project/continuity, complete `current_ask`, and existing approved,
active `constitution_id`. Build returns an artifact, reference and SHA-256; later
calls must forward that same artifact, `artifact_ref` and `expected_digest`.
Canonical cognition and registered instruction sources are recomposed and checked;
changed scope, request, revision, Constitution, rules or rendered bytes are denied.
Rendering occurs after composition. Required scope/request meaning is not silently
clipped into a profile template. The transport has an explicit byte ceiling, not
an assertion about model token accounting.

Writes preserve exact bytes at allowlisted, non-traversing/non-symlink targets.
Receipt commit additionally requires `delivery_target` and verifies its bytes;
replayed keys cannot select a different artifact. These checks prove artifact/source
or exact file delivery only, never current-process refresh or model consumption.
Legacy requests remain template-only. New clients reject legacy success as scoped
proof. The CLI uses the same opt-in fields (`--artifact-version 2`,
`--constitution-id`, `--current-ask`, `--artifact-file`) and propagates HTTP failures.
No Constitution is activated by building, rendering or verifying a preload.

## Prompt hook compatibility

The `before_agent_start` hook treats Pi's event as read-only and returns
`{ systemPrompt }` to override the turn's prompt. Both cache-safe and legacy
layouts use that result; cache telemetry records the returned prompt. Regression
coverage: `node tests/636-readonly-system-prompt.test.mjs`.

## Development feedback and refresh

For the approved development source/runtime, use change → supported native reload/deployment → exact consumer verification. Git push is required only where that project's approved mechanism consumes the remote. Instruction/skill edits need native resource refresh; executable package changes need their supported package/load lifecycle. A file change or reload in another process is not this agent's activation proof. Compiled production daemon changes retain canonical signed release/install controls.

## Loading model

Pi has separate resource types:

- `packages` load executable Pi packages/extensions declared by `package.json`.
- `extensions` load executable TypeScript extension entrypoints.
- `skills` load Agent Skill markdown directories/files such as `SKILL.md`.

Do not register a Focusa `SKILL.md` directory under `packages`; Pi will try to require it as a module and fail. Register the Focusa skill directory under `skills`, or rely on project-local `.pi/skills` discovery from a trusted project root.

## Headless bootstrap behavior

Non-interactive `pi -p` runs are headless and already processing the operator prompt when lifecycle hooks fire. Focusa project-root bootstrap must not call `sendUserMessage()` in that path, because it creates a competing turn and Pi rejects it as already processing.

Current behavior:

- Headless sessions record telemetry and skip project-root bootstrap message injection.
- Interactive sessions with UI may queue the bootstrap as a `followUp` user message.

## Tool result shape

Pi's current `AgentToolResult` shape is:

```ts
{
  content: Array<{ type: "text"; text: string } | { type: "image"; image: string }>;
  details: Record<string, unknown>;
}
```

Focusa tools should put structured `tool_result_v1` data under `details.tool_result_v1`. Do not return the older `structuredContent` shape.

## Registry completeness

When adding a new `FocusaToolFamily`, update all family-indexed registries together. For example, adding `awareness` requires entries in default inputs, next tools, and not-to-use guidance.

## Optional Workpoint mutation enforcement

File mutation enforcement is advisory by default. Set
`FOCUSA_WORKPOINT_MUTATION_ENFORCEMENT=block` (also accepts `enforce` or
`strict`) to make Pi block `write` and `edit` calls outside the active canonical
Workpoint's `target_objects`. Exact file targets and explicitly directory-shaped
targets ending in `/` are admitted. Shell execution fails closed unless the
Workpoint explicitly includes `tool:bash`; an absent Workpoint, missing path, or
empty allowlist also fails closed in strict mode. Blocked calls return a bounded
`focusa.workpoint_mutation_block.v1` receipt and emit the attempted path,
Workpoint, checkpoint, and reason through the daemon Workpoint drift-evidence
route. Widen scope by checkpointing updated targets; never bypass the interceptor.

## Verification

From the repository root or `apps/pi-extension`:

```bash
cd /path/to/focusa/apps/pi-extension
npm run typecheck -- --pretty false

cd /path/to/focusa
PI_SKIP_VERSION_CHECK=1 pi -p --mode text --no-session --no-context-files --no-skills "Reply only: extension load ok"
PI_SKIP_VERSION_CHECK=1 pi -p --mode text --no-session "Reply only: full resource ok"
PI_SKIP_VERSION_CHECK=1 pi -p --mode text --no-session --no-context-files "/skill:focusa\nReply only: focusa skill load ok"
```

Expected results:

- Typecheck passes.
- Extension-only one-shot returns `extension load ok`.
- Full resource one-shot returns `full resource ok`.
- Focusa skill smoke returns `focusa skill load ok`.

## Related operational note

If Pi reports `Cannot find module './init.ts'` from `pi-mcp-adapter`, verify the active global/user package is complete. A stale incomplete global install can shadow a complete user-local package.
