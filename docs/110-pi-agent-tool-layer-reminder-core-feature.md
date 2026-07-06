# Spec 110 — Pi Agent Tool-Layer Reminder Core Feature

Status: Draft  
Owner: Verious Smith  
Created: 2026-06-25  
Scope: Focusa Pi extension runtime, Focusa agent reminder API, `focusa_agent_prompt`, Pi tool execution hooks, optional shell fallback shims, and related docs/skill/tool-contract surfaces.

## 1. Purpose

Focusa is strongest when Pi agents use the canonical `focusa_*` tool layer for Focusa daemon/state work instead of drifting into raw shell, `curl`, `fetch`, or ad hoc HTTP calls.

This spec makes the reminder behavior a **core Focusa feature**:

```text
When an agent is running inside Pi and uses a shell-like tool, Focusa should visibly remind the agent to prefer focusa_* Pi tools for Focusa daemon/state interactions.
```

The feature is advisory, configurable, Pi-conditional, and non-blocking. It should improve agent behavior without breaking shell workflows.

## 1.1 Launch Architecture Adjacency

Spec 110 is a launch-critical guardrail for the surrounding pre-launch architecture specs:

- [Spec 111](111-agent-context-bootstrap-and-delivery-spec.md) depends on Spec 110 reminders so bootstrap/preload work uses the canonical `focusa_*` surfaces.
- [Spec 112](112-install-binary-architecture-audit.md) depends on Spec 110 reminders so installer/platform validation does not drift into uncited shell-only checks.
- [Spec 116](116-provider-neutral-work-item-closure-authority-spec.md) depends on Spec 110 reminders so provider-neutral closure uses governed Focusa authority tools.
- [Spec 117](117-mission-deck-onboarding-recall-pwa-spec.md) depends on Spec 110 reminders so Mission Deck, Recall, and onboarding surfaces preserve Focusa tool authority.
- [Spec 119](119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md) depends on Spec 110 reminders so receipts and governed execution cite canonical tool/evidence surfaces.

The Spec 118 number is not present in the current docs tree; launch readiness must either assign it or explicitly reserve it so the 116–119 architecture chain has no silent gap.

## 2. Source Evidence From Current Repo

This spec is grounded in these current Focusa repo surfaces:

- `crates/focusa-api/src/routes/agent_reminder.rs`
  - Defines `GET /v1/agent/prompt`.
  - Emits structured reminder payloads for Pi agents.
  - Detects Pi clients through `X-Focusa-Client: pi`, `X-Extension-Token: focusa-pi*`, or a `focusa-pi` user agent.
  - Adds `X-Focusa-Agent-Prompt: focusa_*` for detected Pi clients.
  - Injects `_agent_prompt` into JSON bodies or a plain-text reminder trailer when possible.
- `crates/focusa-api/src/server.rs`
  - Merges `routes::agent_reminder::router()` into the API router.
  - Applies `routes::agent_reminder::agent_prompt_response_header_mw` globally.
- `apps/pi-extension/src/state.ts`
  - `focusaFetch()` sends the Pi-identifying headers required by the agent reminder route.
  - Maintains Pi session state, turn state, UI context, telemetry buffers, project identity, Workpoint packet, and Focusa availability.
- `apps/pi-extension/src/polish.ts`
  - Registers Pi lifecycle/tool hooks in `registerPolishHooks`.
  - Tracks `tool_execution_start`, `tool_execution_update`, and `tool_execution_end`.
  - Records bounded hook/token telemetry.
  - Posts best-effort telemetry to Focusa.
- `apps/pi-extension/src/config.ts`
  - Loads `.pi/settings.json`, user settings, and environment overrides.
  - Already provides a strong precedent for Pi runtime feature toggles and env var mapping.
- `apps/pi-extension/src/tools.ts`
  - Registers the `focusa_*` Pi tool surface.
  - Shapes tool responses around `tool_result_v1`, next tools, recovery hints, and bounded visible output.
- `.pi/skills/focusa/SKILL.md`
  - Describes the Focusa cognitive runtime skill, default pickup sequence, tool families, and result-envelope contract.
- `docs/focusa-tools/tools/focusa_agent_prompt.md`
  - Documents `focusa_agent_prompt` as the in-band reminder that Pi clients should use `focusa_*` tools instead of raw `curl`/`fetch` calls.
- `docs/current/HOOK_COVERAGE.md`
  - Lists existing Pi hook coverage, including tool execution hooks.
- `scripts/magic/focusa-magic.sh`
  - Provides harness-wrapper precedent for optional shell/runtime shims.

## 3. Problem Statement

Agents working in Pi often have access to both:

1. General shell-like tools such as bash/zsh/terminal/exec/run-command.
2. Focusa-native `focusa_*` tools.

When a task touches Focusa daemon/state, Workpoints, Focus State, trajectory, evidence, metacognition, or diagnostics, the `focusa_*` tools are usually the correct surface because they preserve Focusa semantics:

- `tool_result_v1` envelopes,
- `failure_class` recovery,
- `next_tools` choreography,
- Workpoint continuity,
- evidence capture/linking,
- project-root/continuity authority,
- bounded output,
- telemetry,
- metacognitive learning,
- and operator-visible recovery guidance.

Raw shell or direct HTTP often bypasses that structure. The agent may technically reach the daemon, but it loses Focusa's canonical affordances and recovery posture.

The current repo already has the API/daemon-side reminder. The missing core feature is runtime visibility at the exact moment drift is likely: shell-like tool usage inside Pi.

## 4. Core Principle

```text
Focusa-aware agents should not merely have Focusa tools available.
They should be continuously nudged back into the canonical Focusa tool layer when their behavior suggests drift.
```

The reminder must be visible enough to help the agent, but bounded enough not to become noise.

## 5. Goals

- Make the Pi shell-tool reminder a named core feature.
- Keep `GET /v1/agent/prompt` and `focusa_agent_prompt` as the canonical reminder source.
- Add Pi-runtime reminder surfacing after shell-like tool execution.
- Make behavior configurable through existing Pi config patterns.
- Keep reminders Pi-conditional by default.
- Avoid daemon reducer/state mutation.
- Avoid blocking or altering shell command execution.
- Add telemetry proving reminder emission without storing raw shell command content by default.
- Document the feature in Focusa docs/skills/tool docs.

## 6. Non-Goals

This spec does not require:

- blocking raw shell commands,
- forbidding `curl`, `fetch`, or shell use,
- replacing `focusa_agent_prompt`,
- making `.zshrc`, `.zshenv`, or `.bashrc` the primary implementation,
- changing every Focusa CLI command,
- adding daemon reducer state,
- automatically creating Workpoints,
- automatically writing Focus State,
- storing raw command text by default,
- or turning UIAI/browser pretests into Focusa failures.

Raw shell remains correct for normal repo operations, package installs, file inspection, git commands, tests, builds, and transport debugging.

## 7. Architecture

Spec 110 has four layers:

```text
Layer 1 — Canonical reminder source
  crates/focusa-api/src/routes/agent_reminder.rs

Layer 2 — Pi runtime shell/tool surfacing
  apps/pi-extension/src/polish.ts

Layer 3 — Pi runtime config/state controls
  apps/pi-extension/src/config.ts
  apps/pi-extension/src/state.ts

Layer 4 — Optional shell fallback
  scripts/magic/focusa-pi-shell-reminder.sh
```

### 7.1 Runtime path

```text
Pi agent uses shell-like tool
        ↓
apps/pi-extension/src/polish.ts receives tool_execution_end
        ↓
tool name is classified as shell-like
        ↓
config/frequency gates approve reminder
        ↓
Pi extension surfaces compact reminder through UI notification when available
        ↓
bounded telemetry records that a reminder fired
        ↓
agent is nudged toward focusa_* tools for Focusa daemon/state work
```

### 7.2 API path

```text
Pi extension calls Focusa API through focusaFetch()
        ↓
Pi-identifying headers are sent
        ↓
agent_reminder middleware detects Pi
        ↓
response includes X-Focusa-Agent-Prompt: focusa_*
        ↓
JSON/plain responses may include a compact in-band reminder
```

## 8. Correct Implementation Location

### 8.1 Primary implementation: Pi extension polish hooks

Primary file:

```text
apps/pi-extension/src/polish.ts
```

This is the correct place because:

- the feature is about Pi agent runtime behavior,
- the hook layer already observes tool execution,
- the reminder is not canonical cognition state,
- the behavior should be non-blocking and local to the Pi runtime,
- existing telemetry patterns live here,
- and shell usage is visible at this layer.

### 8.2 Canonical prompt source: agent reminder route

Primary file:

```text
crates/focusa-api/src/routes/agent_reminder.rs
```

This remains the source of truth for reminder wording/metadata where possible.

### 8.3 Not the reducer/daemon core

Do **not** implement the reminder inside Focusa reducer/daemon cognitive state.

The reminder is runtime UX/AX guidance, not durable Focus State, Workpoint state, or trajectory state.

## 9. Reminder Text

Canonical compact reminder:

```text
🔔 Focusa reminder: For Focusa daemon/state work, prefer focusa_* Pi tools.
Start with: focusa_agent_prompt → focusa_tool_doctor → focusa_workpoint_resume → focusa_evidence_capture.
Avoid raw curl/fetch unless verifying transport, UIAI pretest, or debugging the tool layer itself.
```

Plain ASCII fallback when emoji is undesirable:

```text
Focusa reminder: For Focusa daemon/state work, prefer focusa_* Pi tools.
Start with: focusa_agent_prompt -> focusa_tool_doctor -> focusa_workpoint_resume -> focusa_evidence_capture.
Avoid raw curl/fetch unless verifying transport, UIAI pretest, or debugging the tool layer itself.
```

The reminder must remain short. It is a nudge, not a full skill document.

## 10. Shell-Like Tool Classification

Add helper in:

```text
apps/pi-extension/src/polish.ts
```

```ts
function isShellLikeTool(name: unknown): boolean {
  const toolName = String(name || "").toLowerCase();

  return [
    "bash",
    "zsh",
    "shell",
    "terminal",
    "exec",
    "run_command",
    "command",
  ].some((needle) => toolName === needle || toolName.includes(needle));
}
```

This classifier intentionally errs on the side of visibility. It only triggers a reminder. It does not block or mutate anything.

## 11. Config Model

Add to `FocusaConfig` in:

```text
apps/pi-extension/src/config.ts
```

```ts
agentReminderMode: "off" | "api" | "shell" | "all";
agentReminderShellFrequency: "every_use" | "once_per_turn" | "cooldown";
agentReminderCooldownMs: number;
agentReminderUseEmoji: boolean;
```

Defaults:

```ts
agentReminderMode: "all",
agentReminderShellFrequency: "every_use",
agentReminderCooldownMs: 0,
agentReminderUseEmoji: true,
```

Environment variable mapping:

```text
FOCUSA_PI_AGENT_REMINDER_MODE -> agentReminderMode
FOCUSA_PI_AGENT_REMINDER_SHELL_FREQUENCY -> agentReminderShellFrequency
FOCUSA_PI_AGENT_REMINDER_COOLDOWN_MS -> agentReminderCooldownMs
FOCUSA_PI_AGENT_REMINDER_USE_EMOJI -> agentReminderUseEmoji
```

Validation rules:

```ts
if (!["off", "api", "shell", "all"].includes(cfg.agentReminderMode)) {
  errs.push(`agentReminderMode(${cfg.agentReminderMode}) must be one of: off, api, shell, all`);
}

if (!["every_use", "once_per_turn", "cooldown"].includes(cfg.agentReminderShellFrequency)) {
  errs.push(`agentReminderShellFrequency(${cfg.agentReminderShellFrequency}) must be one of: every_use, once_per_turn, cooldown`);
}

if (cfg.agentReminderCooldownMs < 0) {
  errs.push("agentReminderCooldownMs must be >= 0");
}
```

## 12. State Model

Add to `S` in:

```text
apps/pi-extension/src/state.ts
```

```ts
lastAgentReminderAt: 0,
lastAgentReminderTurn: -1,
```

Reset in `resetPiSessionScopedState()`:

```ts
S.lastAgentReminderAt = 0;
S.lastAgentReminderTurn = -1;
```

No durable Focusa state is written by default.

## 13. Reminder Frequency Gate

Add helper in:

```text
apps/pi-extension/src/polish.ts
```

```ts
function reminderFrequencyAllows(): boolean {
  const cfg = S.cfg as any;
  const frequency = cfg?.agentReminderShellFrequency || "every_use";

  if (frequency === "every_use") return true;

  if (frequency === "once_per_turn") {
    if ((S as any).lastAgentReminderTurn === S.turnCount) return false;
    (S as any).lastAgentReminderTurn = S.turnCount;
    return true;
  }

  if (frequency === "cooldown") {
    const cooldown = Number(cfg?.agentReminderCooldownMs || 0);
    const now = Date.now();
    if (cooldown > 0 && now - Number((S as any).lastAgentReminderAt || 0) < cooldown) return false;
    (S as any).lastAgentReminderAt = now;
    return true;
  }

  return true;
}
```

Add helper:

```ts
function shouldShowAgentToolLayerReminder(toolName: unknown): boolean {
  const cfg = S.cfg as any;
  const mode = cfg?.agentReminderMode || "all";

  if (!S.cfg?.enabled) return false;
  if (mode === "off") return false;
  if (mode !== "all" && mode !== "shell") return false;
  if (!isShellLikeTool(toolName)) return false;

  return reminderFrequencyAllows();
}
```

## 14. Reminder Emission

Add helper in:

```text
apps/pi-extension/src/polish.ts
```

```ts
function focusaAgentToolLayerReminderText(): string {
  const cfg = S.cfg as any;
  const useEmoji = cfg?.agentReminderUseEmoji !== false;
  const prefix = useEmoji ? "🔔 Focusa reminder" : "Focusa reminder";

  return [
    `${prefix}: For Focusa daemon/state work, prefer focusa_* Pi tools.`,
    "Start with: focusa_agent_prompt → focusa_tool_doctor → focusa_workpoint_resume → focusa_evidence_capture.",
    "Avoid raw curl/fetch unless verifying transport, UIAI pretest, or debugging the tool layer itself.",
  ].join("\n");
}
```

Add helper:

```ts
async function emitAgentToolLayerReminder(input: {
  toolName: string;
  toolCallId: string;
  surface: string;
}): Promise<void> {
  const reminder = focusaAgentToolLayerReminderText();

  try {
    const ui = S.uiCtx as any;
    ui?.notify?.(reminder, "info");
  } catch {
    // Reminder must never break shell tool execution.
  }

  recordHookTelemetry({
    hook: "agent_tool_layer_reminder",
    tool_name: input.toolName,
    tool_call_id: input.toolCallId,
    surface: input.surface,
  });

  bestEffortTelemetry("focusa.agent_tool_layer_reminder", {
    source: "pi-extension-spec110",
    tool_name: input.toolName,
    tool_call_id: input.toolCallId,
    surface: input.surface,
    preferred_layer: "focusa_* tools",
  });
}
```

Modify the existing `tool_execution_end` hook:

```ts
hookApi.on("tool_execution_end", async (event: any, _ctx: any) => {
  const id = String(event?.toolCallId || event?.id || "unknown");
  const started = S.spec92ToolStartTimes[id];
  if (started) delete S.spec92ToolStartTimes[id];
  const record = {
    hook: "tool_execution_end",
    tool_call_id: id,
    tool_name: event?.toolName || event?.name || "unknown",
    duration_ms: started ? Date.now() - started : null,
    result_size_bytes: safeJsonSize(event?.result || event),
    status: event?.status || "completed",
  };
  recordHookTelemetry(record);
  bestEffortTelemetry("spec92.tool_execution_end", record);

  if (shouldShowAgentToolLayerReminder(record.tool_name)) {
    await emitAgentToolLayerReminder({
      toolName: String(record.tool_name || "unknown"),
      toolCallId: id,
      surface: "tool_execution_end",
    });
  }
});
```

If Pi hook APIs later allow tool-result mutation, the reminder may also be appended to shell tool results. MVP should only require notification + telemetry to avoid breaking unknown hook contracts.

## 15. Agent Reminder API Enhancement

Enhance `build_prompt()` in:

```text
crates/focusa-api/src/routes/agent_reminder.rs
```

Add field:

```json
{
  "shell_tool_reminder": {
    "enabled_by_default": true,
    "surface": "pi_extension.tool_execution_end",
    "message": "For Focusa daemon/state work, prefer focusa_* Pi tools.",
    "first_tools": [
      "focusa_agent_prompt",
      "focusa_tool_doctor",
      "focusa_workpoint_resume",
      "focusa_evidence_capture"
    ],
    "non_goal": "Does not block shell use; it nudges Focusa daemon/state interactions toward canonical Pi tools."
  }
}
```

Do not make the API route responsible for classifying shell tools. Shell/tool classification belongs to the Pi extension runtime.

## 16. Optional Shell Fallback

Add optional fallback script:

```text
scripts/magic/focusa-pi-shell-reminder.sh
```

Purpose:

- provide degraded reminder coverage when Pi hook visibility is unavailable,
- support explicit operator installation,
- avoid relying on `.zshrc` alone,
- avoid auto-mutating shell startup files.

Script:

```bash
#!/usr/bin/env bash

mode="${FOCUSA_PI_AGENT_REMINDER_MODE:-all}"

if [[ "$mode" == "off" ]]; then
  return 0 2>/dev/null || exit 0
fi

if [[ -n "${FOCUSA_PI_SESSION:-}" || -n "${FOCUSA_PROJECT_ROOT:-}" || "$PWD" == *"/focusa"* ]]; then
  echo "Focusa reminder: Prefer focusa_* Pi tools for Focusa daemon/state work." >&2
  echo "Start with: focusa_agent_prompt -> focusa_tool_doctor -> focusa_workpoint_resume -> focusa_evidence_capture." >&2
fi
```

Install guidance only; do not auto-install without operator approval:

```bash
# zsh broad fallback
source "$HOME/.local/share/focusa/focusa-pi-shell-reminder.sh"

# bash fallback
source "$HOME/.local/share/focusa/focusa-pi-shell-reminder.sh"
```

Caveats:

- `.zshrc` only covers interactive zsh.
- `.zshenv` covers more zsh invocations but can be noisy.
- `bash -lc` does not read `.zshrc`.
- Pi hook surfacing remains the primary implementation.

## 17. Telemetry Contract

When the reminder fires, record bounded telemetry:

```json
{
  "event_type": "focusa.agent_tool_layer_reminder",
  "source": "pi-extension-spec110",
  "payload": {
    "tool_name": "bash",
    "tool_call_id": "abc123",
    "surface": "tool_execution_end",
    "preferred_layer": "focusa_* tools"
  }
}
```

Rules:

- Do not store raw command text by default.
- Do not store full result payloads.
- Do not block on telemetry failure.
- Keep telemetry bounded in the same spirit as Spec92 hook telemetry.

## 18. Safety Rules

The feature must:

- never throw into tool execution,
- never block shell command completion,
- never mutate Focus State automatically,
- never checkpoint Workpoints automatically,
- never store raw shell command text by default,
- never imply shell is forbidden,
- never fire when `agentReminderMode=off`,
- never fire for `focusa_*` tools unless explicitly added later,
- support cooldown/once-per-turn modes,
- and keep UI notification failure non-fatal.

## 19. When Raw Shell Is Still Correct

The reminder must not discourage shell use for ordinary development work:

- `git status`, `git diff`, `git commit`,
- `pnpm install`, `npm install`, `cargo test`,
- file inspection,
- ripgrep/search,
- builds/typechecks,
- service restarts,
- logs,
- transport debugging,
- API proof commands requested by the operator,
- and UIAI/browser pretest workflows.

The reminder is specifically about **Focusa daemon/state work**.

## 20. Docs Updates

Update:

```text
docs/focusa-tools/tools/focusa_agent_prompt.md
```

Add section:

```markdown
## Spec110 shell-tool reminder

Focusa can also surface this reminder after shell-like Pi tool use when `agentReminderMode` is `shell` or `all`.

The shell reminder is advisory only. It does not block shell commands.

Use shell directly when shell is the correct surface. Use `focusa_*` tools when interacting with Focusa daemon/state, Workpoints, Focus State, trajectory, metacognition, diagnostics, or evidence.
```

Update:

```text
docs/current/HOOK_COVERAGE.md
```

Add `agent_tool_layer_reminder` as a derived event emitted from `tool_execution_end`.

Update:

```text
docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md
```

Add:

- config keys,
- behavior summary,
- shell fallback caveat,
- and proof commands.

Update:

```text
.pi/skills/focusa/SKILL.md
```

Add a compact note under default pickup/tool-doctor guidance:

```text
When a Pi agent uses shell for Focusa daemon/state work, prefer focusa_agent_prompt/tool_doctor/workpoint_resume/evidence_capture before raw curl/fetch.
```

## 21. Tool Contract Updates

No new tool is required for MVP.

`focusa_agent_prompt` remains the canonical tool.

Optional future tool:

```text
focusa_agent_reminder_status
```

Family:

```text
awareness
```

Purpose:

```text
Read current agent-reminder config, last reminder time, last shell-like tool seen, and whether Pi headers are being detected.
```

Do not add this optional tool until the core hook behavior is implemented and proven.

## 22. Acceptance Criteria

### 22.1 API reminder still works

Given a request with:

```text
X-Focusa-Client: pi
```

Then response includes:

```text
X-Focusa-Agent-Prompt: focusa_*
```

And:

```http
GET /v1/agent/prompt
```

returns structured agent guidance.

### 22.2 Pi shell reminder fires

Given Pi extension is enabled and config is:

```json
{
  "focusaPiBridge": {
    "agentReminderMode": "all",
    "agentReminderShellFrequency": "every_use"
  }
}
```

When a shell-like tool completes with name:

```text
bash
```

Then Focusa surfaces the compact reminder and records bounded telemetry.

### 22.3 Non-shell tools do not trigger shell reminder

Given tool name:

```text
focusa_workpoint_resume
```

Then no shell-tool reminder is emitted.

### 22.4 Off mode disables reminder

Given:

```text
FOCUSA_PI_AGENT_REMINDER_MODE=off
```

Then no shell-tool reminder is emitted.

### 22.5 Once-per-turn mode works

Given:

```json
{
  "agentReminderShellFrequency": "once_per_turn"
}
```

Then multiple shell-like tool executions in the same turn emit at most one reminder.

### 22.6 Cooldown mode works

Given:

```json
{
  "agentReminderShellFrequency": "cooldown",
  "agentReminderCooldownMs": 60000
}
```

Then repeated shell-like tools within 60 seconds produce at most one reminder.

### 22.7 Reminder cannot break shell tools

If UI notification throws, shell tool execution still completes and telemetry failure is ignored.

### 22.8 Raw command text is not stored by default

Telemetry must include tool name/id/surface, not raw command content, unless a future explicit debug setting enables it.

## 23. Test Plan

### 23.1 Pi extension typecheck

```bash
cd apps/pi-extension
./node_modules/.bin/tsc --noEmit
```

### 23.2 Skill hygiene

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
node scripts/validate-skill-hygiene.mjs
```

### 23.3 Tool contract validation

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
node scripts/validate-focusa-tool-contracts.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

### 23.4 API prompt proof

```bash
curl -i \
  -H 'X-Focusa-Client: pi' \
  -H 'X-Extension-Token: focusa-pi-spec110' \
  http://127.0.0.1:8787/v1/agent/prompt
```

Expected:

```text
X-Focusa-Agent-Prompt: focusa_*
```

### 23.5 Pi hook proof

In Pi, execute a shell-like tool.

Expected visible reminder:

```text
Focusa reminder: For Focusa daemon/state work, prefer focusa_* Pi tools.
```

Then inspect hook/telemetry through:

```text
focusa_tool_doctor scope="spec92"
```

Expected telemetry/hook record includes:

```text
agent_tool_layer_reminder
```

## 24. Implementation Plan

### Phase 1 — Spec accepted

Files:

```text
docs/110-pi-agent-tool-layer-reminder-core-feature.md
```

Work:

- Create this spec.
- Treat implementation as blocked until tasks are decomposed according to spec-first lifecycle rules.

### Phase 2 — Core Pi hook

Files:

```text
apps/pi-extension/src/polish.ts
apps/pi-extension/src/state.ts
apps/pi-extension/src/config.ts
```

Work:

- Add config fields and env mapping.
- Add session state fields.
- Add shell-like tool classifier.
- Add frequency gate.
- Emit UI notification and telemetry from `tool_execution_end`.

### Phase 3 — API prompt metadata

File:

```text
crates/focusa-api/src/routes/agent_reminder.rs
```

Work:

- Add `shell_tool_reminder` metadata to structured prompt.
- Keep existing response header/body behavior.

### Phase 4 — Docs/skill sync

Files:

```text
docs/focusa-tools/tools/focusa_agent_prompt.md
docs/current/HOOK_COVERAGE.md
docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md
.pi/skills/focusa/SKILL.md
```

Work:

- Document config keys.
- Document runtime behavior.
- Document fallback caveats.
- Document raw-shell exception cases.

### Phase 5 — Optional shell fallback

Files:

```text
scripts/magic/focusa-pi-shell-reminder.sh
scripts/magic/install.sh
```

Work:

- Add fallback helper.
- Add opt-in install instructions.
- Do not auto-edit shell startup files without explicit operator approval.

## 25. Definition of Done

Spec 110 is complete when:

- the spec file exists in `docs/`,
- Pi agents receive visible reminders after shell-like tool usage,
- reminders are configurable and Pi-conditional,
- reminder telemetry is bounded and command-content-safe,
- API prompt remains canonical,
- `focusa_agent_prompt` docs describe both API and shell-tool reminder paths,
- hook coverage docs mention the derived reminder event,
- the feature can be disabled with one config/env setting,
- and the validation/typecheck proof commands pass.

## 26. Final Positioning

Spec 110 makes Focusa more agentic by protecting its most important runtime behavior:

```text
Agents should use Focusa's canonical cognition/tool layer when working with Focusa state.
```

The feature does not reduce shell power. It makes shell use safer by reminding Pi agents that Focusa daemon/state work has a richer, safer, more recoverable surface: the `focusa_*` tools.
