# Focusa Awareness Plugin

Standalone OpenClaw-compatible plugin that injects a Focusa Utility Card through the public `before_prompt_build` hook.

## Build

```bash
cd apps/focusa-awareness
npm run check
npm run build
```

The plugin intentionally declares a minimal local host API interface instead of importing `openclaw/plugin-sdk`; this keeps the package buildable in this repository while preserving the expected runtime hook shape.

## Hook compatibility and regression history

The November 2026 repair replaces `before_agent_start`, an unrecognized typed hook in both OpenClaw 2026.10.7 and 2026.11.3. This is a pre-existing adapter defect, not a new incompatibility introduced by upgrading. Hosts can report the plugin loaded while ignoring its obsolete hook.

Both reviewed releases support `before_prompt_build` and its `{ prependContext }` result. Register exactly one hook: dual registration can inject duplicate context. The change preserves fetching, timeout, scope checks, degraded fallback, and disabled behavior. Compatibility claims cover these reviewed contracts, not every past or future host. Re-run qualification when upgrading OpenClaw.

Non-bundled plugins also require `plugins.entries.focusa-awareness.hooks.allowConversationAccess: true`; enable only for this approved adapter. A configured folder is not verified project/task authority, and a missing continuity identifier must never be invented. Unbound/degraded cards remain advisory.

The package explicitly declares `openclaw.extensions: ["./dist/index.js"]`, matching its `main` entry. Build with Node 24.22.1 or a compatible runtime exposing `node:module.stripTypeScriptTypes`; no Bun installation is needed. Explicit artifact selection avoids falling back to a stale cached `index.ts` module during a same-process hot replacement.

Run `npm test` on a supported Node runtime; this builds and tests the actual declared JavaScript entry point. Fixtures retain release source references and hashes; tests check one recognized registration, one fetch/injection, disabled behavior, missing scope, degraded fetch handling, and the declared artifact path. These isolated tests are not proof of an actual host load or a future release.

For an approved hot deployment, preserve exact source/config preimages with ownership and hashes; change only the necessary hook and permission; use the native plugin reload; then verify the loaded generation and a real prompt preparation. Keep canonical-source and legacy-deployment differences explicit instead of copying unrelated behavior changes. Roll back only the changed files/field and reload again, preserving concurrent config changes. No host package upgrade or full-service restart is implied.

## Runtime config

See `openclaw.plugin.json` for supported config fields:

- `focusaUrl` — Focusa daemon URL, default `http://127.0.0.1:8787`.
- `adapterId`, `workspaceId`, `agentId`, `operatorId` — adapter identity echoed into the awareness card.
- `projectRoot` — verified project root passed as `project_root`.
- `continuityId` — stable logical workstream id passed as `continuity_id`.
- `timeoutMs` — awareness request timeout.
- `enabled` — disable injection when false.

## Scope and authority contract

Adapter surfaces must preserve Focusa scope semantics:

1. Pass `project_root` and `continuity_id` whenever known.
2. Treat `project_root + continuity_id` as the Workpoint authority boundary.
3. Treat Workpoint resume/checkpoint as immediate action authority.
4. Treat Trajectory and awareness cards as advisory unless the response explicitly says `canonical=true` for the verified scope.
5. Preserve labels such as `canonical`, `advisory`, `degraded`, `stale`, `scope_status`, and `scope_conflict_reason` in any UI or log surface.

If `continuityId` is not configured, the card is allowed to be advisory/unbound. Do not promote it to canonical task authority.

## Tool-result pass-through expectations

The `/v1/awareness/card` response may include structured status fields and `details.tool_result_v1`-style payloads. OpenClaw adapters should pass these through without flattening away:

- `status`, `failure_class`, `canonical`, `degraded`, `stale`
- `scope.project_root`, `scope.continuity_id`, `scope.scope_status`
- `evidence_refs`, `side_effects`, `next_tools`
- `retry.safe` and `retry.posture`

This aligns the plugin with `docs/current/AGENT_ADAPTER_CONTRACT.md`: UI copy may be concise, but machine-readable scope/authority labels must remain inspectable.

## Degraded fallback

If the daemon is unreachable, the plugin injects a degraded fallback card. The fallback includes configured `project_root` and `continuity_id` but must be treated as `cognition_degraded=true`; fetch a canonical Workpoint resume before risky continuation.
