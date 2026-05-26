# Focusa Feature Maturity Audit — 2026-05-26

Scope: static code/readme audit of `crates/*`, `apps/*`, `packages/*`, `tests/*`, and current docs. Ratings are 1–10 maturity estimates for current software usefulness, not final product quality.

Evidence snapshot:
- Rust files: 181 under `crates/`
- API route files: 53 under `crates/focusa-api/src/routes/`
- CLI command files: 38 under `crates/focusa-cli/src/commands/`
- Pi tools: 59 registered/documented tools
- Tests: 303 shell/TS/runtime/static gates under `tests/`
- Menubar app: Svelte/Tauri slice under `apps/menubar/`
- Pi extension: `apps/pi-extension/src/tools.ts`, `compaction.ts`, `state.ts`, `commands.ts`
- Focused audit run after session-transfer parity returned 62 tools; TypeScript, JSON projection, and public registry docs are synchronized.
- Prediction/metacognition flywheel pass added `capture-outcome`, ontology context, prediction→metacog capture, metacog→follow-up prediction, and `tests/spec98_prediction_metacog_flywheel_static_test.sh`.

## Summary: most underdeveloped/high-leverage areas

| Priority | Feature area | Rating | Why it matters now |
| --- | ---: | ---: | --- |
| 1 | Autonomic coding-work governor | 2/10 | Not yet a first-class feature; would directly improve continuous coding agents by reading project vitals and regulating checkpoint/compact/retry/stop decisions. |
| 2 | Coding-specific project vitals/sensory layer | 3/10 | Focusa has telemetry, project identity, and work-loop health, but no single normalized card for git/tests/lint/CI/logs/beads/ownership. |
| 3 | Loop/stuck detector | 3/10 | Recovery primitives exist, but repeated failed commands, same-file churn, and investigation loops are not clearly detected as workflow states. |
| 4 | Sleep/consolidation cycle | 4/10 | Checkpoint/compaction pieces exist, but scheduled consolidation/prune/promote is not yet a cohesive loop. |
| 5 | Menubar/Tauri cockpit | 5/10 | Good read-only cockpit foundation; packaging/security, tray status, write flows, and drawers remain incomplete. |
| 6 | Cross-agent adapter parity | 5/10 | Pi is strong; non-Pi docs/plugin exist, but Claude/OpenCode/Letta/MCP cards are thinner than Pi. |
| 7 | Outcome-driven maturity scoring | 5/10 | Prediction/metacog flywheel now captures outcomes and follow-up learning, but feature-level rollups are still not automatic. |

## Feature maturity table

| Feature / subsystem | Rating | Evidence in code/docs | What is developed | Main gap |
| --- | ---: | --- | --- | --- |
| Core runtime daemon/state reducer | 8 | `crates/focusa-core/src/reducer.rs`, `runtime/daemon.rs`, `runtime/persistence_sqlite.rs`; `crates/focusa-api/src/server.rs` | Large reducer-backed runtime, persistence, daemon loops, event bus, API state. | Still active-dev; broad reducer complexity and some route/test-only unwraps require continued hardening. |
| API surface | 8 | 53 route files; `crates/focusa-api/src/routes/*.rs`; `API_REFERENCE_CURRENT.md` | Broad HTTP API for focus, workpoints, trajectory, metacog, telemetry, events, resource, ontology, etc. | Route count is high; product paths need simplification around coding-agent workflows. |
| CLI surface | 7 | 38 command files; `crates/focusa-cli/src/commands/*.rs`; `CLI_REFERENCE_CURRENT.md` | Broad CLI parity for core runtime features. | UX still expert-oriented; workflows require knowing many commands. |
| Pi extension tool integration | 8 | `apps/pi-extension/src/tools.ts`, 62 tools, focused docs; tool wrapper/envelopes | Strongest agent integration; tool envelopes, recovery hints, compaction packets, project scope. | Contract registry now includes project-card outcome and session-transfer parity; Pi-first parity still needs continued cross-agent visibility. |
| Tool contract registry/parity | 8 | `docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md`; audit script | 62 tools tracked; API/CLI/docs parity intended. | Registry/docs projection synchronized; keep live proof after daemon rebuild/restart. |
| Workpoint continuity | 9 | `routes/workpoint.rs`, `commands/workpoint.rs`, Pi tools, `WORKPOINT_LIFECYCLE_GUIDE.md` | Canonical checkpoint/resume/handoff contract; project_root + continuity gates; evidence linking. | Could be more automatic for coding-agent vitals and periodic consolidation. |
| Trajectory/project north-star | 7 | `routes/trajectory.rs`, `commands/trajectory.rs`, `TRAJECTORY_GTM_AND_GAPS.md` | Per-project HLT/MLG/STG/Waypoint concepts and APIs. | Lifecycle quality metrics and obvious end-user flows still early. |
| Project identity/scope guard | 8 | `routes/project.rs`, Workpoint guards, `WORKPOINT_SESSION_SCOPE_GUARD.md` | Strong project_root/continuity_id safety and cross-project rejection. | Needs tighter integration into coding vitals and all non-Pi adapters. |
| Focus State / scratchpad separation | 8 | `routes/focus.rs`, Pi Focus State tools, `focusa_scratch`/`focusa_decide` validation | Bounded durable state plus scratchpad separation; validation prevents pollution. | Focus State can still become stale/noisy without smarter consolidation. |
| Focus Stack / Focus Gate | 6 | `focus/stack.rs`, `gate/focus_gate.rs`, `routes/gate.rs` | Core attention mechanics and APIs exist. | Salience scoring is not yet clearly outcome-trained or coding-signal driven. |
| Intuition/subconscious layer | 5 | `intuition/engine.rs`, `routes/capabilities_extra.rs`, Doc78 audit | Signals/pattern routes and bounded secondary cognition concepts exist. | Prior audit says broad structured subconscious layer is not mature. |
| Work-loop continuous execution | 7 | `routes/work_loop.rs`, `scripts/work_loop_watchdog.sh`, Pi work-loop tools | Writer ownership, status/health/control/context/checkpoint/select-next. | Not yet an autonomic coding-work governor with test/git/CI/stuck regulation. |
| Reflex primitives/recovery | 8 | `routes/reflex.rs`, Spec97 docs/tests, Pi `reflex_suggestions` | Recovery primitives and no-deadend hints implemented. | Mostly recovery-oriented; positive execution reflexes and workflow reflexes underdeveloped. |
| Tool result envelopes/no-deadends | 8 | `middleware/error_envelope.rs`, Pi wrapper, `TOOL_RESULT_ENVELOPE_V1.md` | Structured failure classes, retry posture, next tools, degraded/canonical semantics. | Needs more per-tool examples and learned routing weights. |
| Evidence/proof refs | 8 | `routes/workpoint.rs`, `visual_workflow.rs`, evidence docs | Stable evidence refs and Workpoint linkage. | More automatic capture from tests/CI/logs would improve coding workflows. |
| Metacognition loop | 8 | `routes/metacognition.rs`, `commands/metacognition.rs`, focused docs | Capture/retrieve/reflect/adjust/evaluate, recent readbacks, promotion scoring, prediction follow-up records. | Needs richer learned outcome metrics and Focus Slice surfacing to reach 9. |
| Prediction/predictive power | 8 | `routes/predictions.rs`, `commands/predict.rs`, `PREDICTIVE_POWER_GUIDE.md` | Record/recent/evaluate/stats, capture-outcome, trajectory/ontology context, prediction→metacog capture. | Needs runtime dogfood, dataset/feed helper tooling, and Focus Slice `PREDICTIVE_CONTEXT` to reach 9. |
| Tree/lineage/snapshots | 7 | `clt/mod.rs`, `routes/snapshots.rs`, `routes/clt.rs`, lineage commands | Branch-aware lineage and snapshot helpers. | UX is tool-heavy; restore/diff flows need more real-world coding-agent proof. |
| Bounded traversal/ontology hot paths | 7 | `routes/traverse.rs`, `routes/ontology.rs`, ontology docs | Bounded traversal and ontology surfaces with low-memory posture. | Runtime ontology-to-action bridge still feels specialist, not simple product. |
| Telemetry/resource/homeostasis | 6 | `routes/telemetry.rs`, `routes/resource.rs`, `EFFICIENCY_GUIDE.md` | Memory/token/cache/tool telemetry and LowMem/resource mode. | No unified “project physiology” card for token/memory/daemon/test/CI pressure. |
| Daemon resilience/low-resource hardening | 7 | `server.rs` monitor/prune loops, `DAEMON_RESILIENCE.md`, Spec96 tests | LowMem fallback and hot/cold payload gates. | Needs production SLO dashboards and automatic recovery policy clarity. |
| Events/SSE/observability | 6 | `routes/events.rs`, `events_sqlite.rs`, `sse.rs`, telemetry routes | Recent/stream/get routes and event persistence paths. | Not yet surfaced as a simple workflow narrative for agents/operators. |
| Autonomy/capabilities/permissions | 5 | `routes/autonomy.rs`, `capabilities.rs`, `permissions.rs`, docs 12–26 | Design and routes exist for autonomy/capability governance. | Real user-facing earned autonomy workflows appear less mature than continuity tools. |
| Constitution/proposals/governance | 5 | `constitution/mod.rs`, `routes/constitution.rs`, `routes/proposals.rs`, docs 16/41 | Proposal and constitution concepts/routes exist. | Likely design-rich, product-light; not central to immediate coding-agent ROI. |
| Training/export/contribution | 4 | `training/mod.rs`, `routes/training.rs`, `commands/export.rs`, `contribute.rs` | Schemas/routes/commands exist. | Appears peripheral; lower visible runtime/product maturity. |
| Sync/multi-device | 4 | `sync/*`, `routes/sync*.rs`, menubar SyncPanel | CRDT/sync routes and UI slice exist. | Multi-device production story and peer UX still underdeveloped. |
| Menubar/Tauri cockpit | 5 | `apps/menubar/src/lib/components/*`, `TAURI_MENUBAR_IMPLEMENTATION_GAPS.md` | Cockpit cards and read-only peeks exist; shared API client. | Packaging/security/tray status/write flows/drawers incomplete. |
| TUI | 3 | `crates/focusa-tui`, some `unwrap()` in views | TUI crate/views exist. | Looks weaker than CLI/Pi/menubar; unwraps in views imply fragile UX. |
| Visual workflow support | 5 | `routes/visual_workflow.rs`, visual UI docs | Visual evidence storage/listing and UI docs exist. | Narrow slice; needs stronger end-to-end visual task loops. |
| Non-Pi agent awareness | 5 | `apps/focusa-awareness/index.ts`, `NON_PI_AGENT_FOCUSA_USAGE.md`, Spec93 | Awareness plugin/docs exist. | Thin compared with Pi extension; less rich tool/compaction integration. |
| Friendly onboarding/agent utility | 6 | `commands/onboard.rs`, `FOCUSA_AGENT_UTILITY_CARD.md`, onboarding docs | Good docs and command-center surfaces. | Install/demo path still expert-oriented. |
| Release/proof automation | 7 | `scripts/release.sh`, `routes/release.rs`, proof docs/tests | Release prove and static/runtime gates exist. | Manual proof remains in places; CI/product packaging gaps remain. |
| Autonomic coding-work governor | 2 | No dedicated route/command/doc before current discussion; adjacent work-loop/resource/telemetry pieces exist | Concept now identified. | Needs first-class project vitals, stuck detection, consolidation, and action regulation. |
| Coding-specific sensory/project vitals | 3 | Project identity, telemetry, Workpoint, Beads conventions external; no unified vitals route | Raw pieces exist. | Need normalized git/tests/lint/CI/logs/beads/file-owner card. |
| Loop/stuck detector | 3 | Reflexes and failure classes exist; no explicit loop detector found | Recovery metadata can support it. | Need repeated-command/same-file/failing-test/churn detection and strategy shift. |
| Sleep/consolidation cycle | 4 | Workpoint checkpoint, compaction fallbacks, metacog capture | Manual/triggered pieces exist. | Need scheduled consolidation that summarizes, prunes, promotes lessons, updates next atomic task. |
| Outcome-driven feature maturity scoring | 5 | Prediction/metacog/evidence pieces plus flywheel outcome capture | Outcomes can feed learning and follow-up predictions. | No automated feature maturity rollup from code coverage, tests, proof, incidents, and user value. |

## Recommended immediate build order

1. Keep tool contract registry drift green through static validation and live proof after each new Pi tool.
2. Add `project_vitals` / coding sensory card: git status, changed files, tests/lint/typecheck state, CI state, bead state, daemon/resource status, Workpoint/Trajectory state.
3. Add loop/stuck detector: repeated command failures, same-file churn, repeated investigation, repeated tool fallback, stale branch/CI red.
4. Add autonomic coding-work governor: if vitals show pressure or stuckness, choose checkpoint/compact/narrow/retry/escalate/stop-unsafe.
5. Add consolidation cycle: after N turns/time/changed-files/tests, write Workpoint checkpoint + evidence refs + lesson candidates + next atomic task.
6. Expand non-Pi adapter cards to consume the same vitals/governor output.
7. Add maturity scoreboard generated from tests, contracts, docs, evidence, runtime probes, and prediction/metacog outcomes.

