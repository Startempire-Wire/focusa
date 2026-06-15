# Pi Extension Toolset Audit Snapshot

Static/tooling gate for Spec106 Vision Tightening: reevaluate Pi plugin tools, cards, post-compaction recovery, and auto-bootstrap after the implementation beads. This is not native menubar or real browser/product QA final signoff.

## Surfaces reviewed

| Surface | Files | Audit result |
| --- | --- | --- |
| Pi tool registry/contracts | `apps/pi-extension/src/tools.ts`, `apps/pi-extension/src/tool-contracts.ts`, `docs/current/focusa-tool-contracts.json`, `tests/pi_extension_contract_test.sh` | retained; tools remain mapped to Golden Workflow families, contracts validate, and saturated live surfaces pass via typed degraded envelopes |
| Utility/agent cards | `apps/pi-extension/src/awareness.ts`, `apps/pi-extension/prompts/focusa-context.md` | tightened route hint; preserves HLT/MLG/STG/Waypoints/Workpoint and authority/advisory labels |
| Post-compaction cards | `apps/pi-extension/src/compaction.ts`, `apps/pi-extension/src/state.ts` | retained; uses Attention Recall, current ask scope verdict, Workpoint/Trajectory packets, and exact-scope rejection |
| Auto-bootstrap/session | `apps/pi-extension/src/session.ts`, `apps/pi-extension/src/state.ts` | retained; verifies project scope, refreshes trajectory/workpoint, and rejects stale/mismatched packets |
| Skills/agent guidance | `apps/pi-extension/skills/focusa/SKILL.md`, `apps/pi-extension/skills/focusa-workpoint/SKILL.md`, `apps/pi-extension/skills/focusa-cli-api/SKILL.md` | retained; guidance routes through Workpoint/Trajectory/Context Authority/Evidence |
| Docs/current surfaces | `GOLDEN_WORKFLOW.md`, `AUTHORITY_MODEL.md`, `AGENT_ADAPTER_CONTRACT.md`, `FOCUSA_GLOSSARY_LINKED_DOCS_UI.md` | updated across Spec106 slices and linked from final audit |

## Redundant/noisy calls removed or justified

- Removed: one overlong Utility Card route sentence that repeated every tool family path.
- Justified: project verify → trajectory → Workpoint bootstrap remains necessary to avoid scope drift.
- Justified: post-compaction Attention Recall + Current Ask Scope verdict remains necessary after tool-output flood or project-switch risk.
- Justified: Context Cognition remains advisory and is not substituted for Workpoint authority.
- Justified: Prediction/metacognition stay end-of-task/evaluation loops, not every-turn mandatory calls.
- Tightened: strict Pi contract now treats `daemon_unavailable` / `resource_exhausted` `tool_result_v1` envelopes as valid degraded contract behavior when saturated.

## Authority compliance

- Authority-bearing cards must carry exact `project_root + continuity_id` scope.
- Session id is temporal metadata only.
- Workpoint is immediate continuation authority only when canonical and scope-matched.
- Trajectory is north-star route context.
- Context Cognition, Project Card, Prediction, and Metacognition are advisory unless linked through Workpoint/Trajectory/Evidence.
- Degraded/advisory packets are never rendered as canonical continuation truth.

## Exact-scope rejection evidence

- `isWorkpointPacketScopedToCurrentSession` compares `project_root`, `continuity_id`, and `session_id` where required.
- `getScopedWorkpointPacket` filters cached packets before Utility Card rendering.
- `session.ts` normalizes and stamps Workpoint resume packets after live resume.
- `compaction.ts` uses scoped packet checks before injecting post-compaction continuation context.

## Final proof commands

```bash
cd apps/pi-extension && npm run check
tests/pi_extension_final_toolset_audit_static_test.sh
tests/pi_extension_contract_test.sh
node scripts/validate-focusa-tool-contracts.mjs
scripts/generate-tool-surface-summary --check
scripts/verify-doc-version-consistency
```

## Completion rule

This audit can satisfy the Pi-extension/tooling slice of Spec106 only. Overall product readiness still depends on real browser/product QA evidence, including the open menubar/native validation boundaries tracked by `focusa-qasy`, `focusa-qasy.25`, and `focusa-ui0y.15`.
