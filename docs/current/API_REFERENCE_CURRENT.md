# Current API Route Inventory

Generated from current `crates/focusa-api/src/routes/*.rs` route registrations. This is an inventory, not a full schema reference.

### ascc
- `GET /v1/ascc/state`
- `GET /v1/ascc/frame/{frame_id}`
- `POST /v1/ascc/update-delta`

### attachments
- `POST /v1/attachments/attach`
- `POST /v1/attachments/detach`
- `GET /v1/attachments/list`

### autonomy
- `GET /v1/autonomy`
- `GET /v1/autonomy/history`

### capabilities
- `GET /v1/agents`
- `GET /v1/agents/{agent_id}`
- `GET /v1/state/current`
- `GET /v1/state/history`
- `GET /v1/state/stack`
- `GET /v1/state/diff`
- `GET /v1/lineage/head`
- `GET /v1/lineage/tree`
- `GET /v1/lineage/node/{clt_node_id}`
- `GET /v1/lineage/path/{clt_node_id}`
- `GET /v1/lineage/children/{clt_node_id}`
- `GET /v1/lineage/summaries`
- `GET /v1/references`
- `GET /v1/references/search`
- `GET /v1/references/{ref_id}`
- `GET /v1/references/{ref_id}/meta`

### capabilities_extra
- `GET /v1/autonomy/status`
- `GET /v1/autonomy/ledger`
- `GET /v1/autonomy/explain`
- `GET /v1/gate/policy`
- `GET /v1/gate/scores`
- `GET /v1/gate/explain`
- `GET /v1/intuition/signals`
- `GET /v1/intuition/patterns`
- `POST /v1/intuition/submit`
- `GET /v1/metrics/uxp`
- `GET /v1/metrics/ufi`
- `GET /v1/metrics/perf`
- `GET /v1/metrics/session/{session_id}`
- `GET /v1/cache/status`
- `GET /v1/cache/policy`
- `GET /v1/cache/events`
- `GET /v1/contribute/status`
- `GET /v1/contribute/policy`
- `GET /v1/contribute/queue`
- `GET /v1/export/history`
- `GET /v1/export/manifest/{export_id}`
- `GET /v1/constitution/diff`
- `GET /v1/constitution/drafts`
- `POST /v1/constitution/propose`
- `GET /v1/state/explain`
- `GET /v1/references/salient`
- `GET /v1/references/trace`
- `GET /v1/telemetry/process`
- `GET /v1/telemetry/ux`

### clt
- `GET /v1/clt/nodes`
- `GET /v1/clt/path`
- `GET /v1/clt/stats`

### commands
- `POST /v1/commands/submit`
- `GET /v1/commands/status/{command_id}`
- `GET /v1/commands/log/{command_id}`

### constitution
- `GET /v1/constitution/active`
- `GET /v1/constitution/versions`
- `POST /v1/constitution`
- `POST /v1/constitution/load`

### ecs
- `GET /v1/ecs/handles`
- `POST /v1/ecs/store`
- `GET /v1/ecs/resolve/{handle_id}`
- `GET /v1/ecs/content/{handle_id}`
- `POST /v1/ecs/rehydrate/{handle_id}`

### env
- `GET /v1/env`

### events
- `GET /v1/events/recent`
- `GET /v1/events/stream`
- `GET /v1/events/{event_id}`

### events_sqlite
- `GET /v1/events/recent`
- `GET /v1/events/{event_id}`

### events_stream
- `GET /v1/events/stream`

### focus
- `GET /v1/focus/stack`
- `GET /v1/focus/frame/current`
- `POST /v1/focus/push`
- `POST /v1/focus/pop`
- `POST /v1/focus/set-active`
- `POST /v1/focus/update`
- `GET /v1/focusa/enabled`
- `PATCH /v1/focusa/enabled`

### gate
- `GET /v1/focus-gate/candidates`
- `POST /v1/focus-gate/suppress`
- `POST /v1/focus-gate/pin`
- `POST /v1/focus-gate/surface`
- `POST /v1/focus-gate/ingest-signal`
- `POST /v1/gate/signal`

### health
- `GET /v1/health`
- `GET /v1/doctor`

### info
- `GET /v1/info`

### instances
- `POST /v1/instances/connect`
- `POST /v1/instances/disconnect`
- `GET /v1/instances/list`

### memory
- `GET /v1/memory/semantic`
- `POST /v1/memory/semantic/upsert`
- `GET /v1/memory/procedural`
- `POST /v1/memory/procedural/reinforce`

### metacognition
- `POST /v1/metacognition/capture`
- `POST /v1/metacognition/retrieve`
- `POST /v1/metacognition/reflect`
- `GET /v1/metacognition/reflections/recent`
- `POST /v1/metacognition/adjust`
- `GET /v1/metacognition/adjustments/recent`
- `POST /v1/metacognition/evaluate`

### ontology
- `GET /v1/ontology/primitives`
- `GET /v1/ontology/contracts`
- `GET /v1/ontology/world`
- `GET /v1/ontology/slices`
- `GET /v1/ontology/tool-contracts`
- `POST /v1/ontology/actions`

### proposals
- `GET/POST /v1/proposals`
- `POST /v1/proposals/resolve`

### proxy
- `POST /proxy/v1/chat/completions`
- `POST /proxy/v1/messages`
- `POST /proxy/acp`

### reflection
- `POST /v1/reflect/run`
- `GET /v1/reflect/history`
- `GET /v1/reflect/status`
- `POST /v1/reflect/scheduler/tick`

### rfm
- `GET /v1/rfm`

### session
- `GET /v1/status`
- `GET /v1/state/dump`
- `POST /v1/session/start`
- `POST /v1/session/resume`
- `POST /v1/session/close`

### skills
- `GET /v1/skills`

### snapshots
- `POST /v1/focus/snapshots`
- `GET /v1/focus/snapshots/recent`
- `POST /v1/focus/snapshots/restore`
- `POST /v1/focus/snapshots/diff`

### sse
- `GET /v1/events/stream`
- `GET /v1/events/health`

### sync
- `GET/POST /v1/sync/peers`
- `DELETE /v1/sync/peers/{peer_id}`
- `GET /v1/sync/status/{peer_id}`
- `POST /v1/sync/pull/{peer_id}`
- `POST /v1/sync/push/{peer_id}`
- `POST /v1/sync/receive`
- `POST /v1/sync/transfer`

### telemetry
- `GET /v1/telemetry/tokens`
- `GET /v1/telemetry/token-budget/status`
- `POST /v1/telemetry/token-budget`
- `GET /v1/telemetry/cache-metadata/status`
- `POST /v1/telemetry/cache-metadata`
- `GET /v1/telemetry/cost`
- `GET /v1/telemetry/tools`
- `POST /v1/telemetry/tool-usage`
- `POST /v1/telemetry/activity`
- `POST /v1/telemetry/ops`
- `POST /v1/telemetry/event`
- `POST /v1/telemetry/trace`
- `GET /v1/telemetry/trace`
- `GET /v1/telemetry/trace/stats`

### threads
- `GET/POST /v1/threads`
- `GET /v1/threads/{id}`
- `POST /v1/threads/{id}/fork`
- `POST /v1/threads/{id}/transfer`

### tokens
- `POST /v1/tokens/create`
- `POST /v1/tokens/revoke`
- `GET /v1/tokens/list`

### training
- `GET /v1/export/status`
- `POST /v1/export/run`
- `GET /v1/training/status`
- `POST /v1/contribute/enable`
- `POST /v1/contribute/pause`
- `POST /v1/contribute/approve`
- `POST /v1/contribute/submit`

### trust
- `PATCH /v1/trust/metrics`

### turn
- `POST /v1/turn/start`
- `POST /v1/turn/append`
- `POST /v1/turn/complete`
- `POST /v1/prompt/assemble`

### uxp
- `GET /v1/uxp`
- `GET /v1/ufi`

### visual_workflow
- `GET /v1/visual-workflow/evidence`

### work_loop
- `GET /v1/work-loop`
- `GET /v1/work-loop/status`
- `POST /v1/work-loop/enable`
- `POST /v1/work-loop/pause`
- `POST /v1/work-loop/resume`
- `POST /v1/work-loop/select-next`
- `POST /v1/work-loop/context`
- `POST /v1/work-loop/driver/start`
- `POST /v1/work-loop/driver/prompt`
- `POST /v1/work-loop/driver/abort`
- `POST /v1/work-loop/driver/stop`
- `POST /v1/work-loop/session/attach`
- `POST /v1/work-loop/session/abort`
- `POST /v1/work-loop/events`
- `POST /v1/work-loop/pause-flags`
- `POST /v1/work-loop/delegation/enable`
- `POST /v1/work-loop/degraded`
- `GET /v1/work-loop/checkpoints`
- `POST /v1/work-loop/checkpoint`
- `POST /v1/work-loop/heartbeat`
- `POST /v1/work-loop/stop`

### workpoint
- `POST /v1/workpoint/checkpoint`
- `GET /v1/workpoint/current`
- `POST /v1/workpoint/resume`
- `POST /v1/workpoint/active-object/resolve`
- `POST /v1/workpoint/evidence/link`
- `POST /v1/workpoint/drift-check`
