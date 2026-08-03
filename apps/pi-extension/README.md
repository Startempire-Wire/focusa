# Focusa Pi Extension

Focusa Pi Bridge is the Pi coding-agent extension that registers Focusa tools, lifecycle hooks, compaction/session handling, and Focusa skills used by Pi sessions.

## v0.9.142 surface

The extension exposes the 135-tool parity surface through typed project/workstream scope. Startup/reload now renders the Focusa Utility Card; recovery composes `focusa_tool_doctor`, project identity, Trajectory, Workpoint checkpoint/resume, evidence capture/linking, prediction evaluation, metacognition, and work-loop routes. Context Cognition, Project Card/Genesis, Temporal Authority, preload, Silent Sessions, browser/UIAI workflows, device pairing, and Tool Discovery remain available through the same scope and operator-steering boundary.

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

## Mission Canvas in the current Pi terminal

Mission Canvas is mounted by this TypeScript extension through `ctx.ui.custom()`; it must not launch a browser, webview, sidecar, or remote host.

```text
/mission-canvas on      # replace only the visible Pi root
/mission-canvas off     # restore stock Pi in the same session
/mission-canvas toggle
```

After changing installed extension sources, run `/reload` in the current Pi session, then toggle Canvas on. A safe local update copies the existing `src/` files to `~/.pi/agent/extension-backups/<timestamp>/` before replacing them. If validation fails, restore that backup and run `/reload` again. The worktree remains the source of truth; never edit or commit the main worktree for a feature-branch Canvas update.

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
