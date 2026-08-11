# Current API Route Inventory

Generated from current Axum route registration plus the Spec135/Spec141 operation registry. This public inventory is release-gated; do not edit route rows manually.

- Classified paths: `545`
- Agent eligible: `114`
- Operator only: `419`
- Public health/pairing: `6`
- Internal: `6`

## Release-current architecture

Exact authority is `project_root + continuity_id`; worktrees are typed working subpaths. Agent discovery is progressive through the Agent Card, tool search/describe/graph/bundle, and strict schemas. Silent Sessions are daemon-native. Mission Canvas and Work Rail bind scoped Work Surfaces, connectors, domain projections, UIAI context, and adaptive generated UI to canonical operations.

Machine authority: [`route-classification.json`](../contracts/spec141/generated-capability-v2/route-classification.json), [`rest-agent-operations.json`](../contracts/spec141/generated-capability-v2/rest-agent-operations.json), and [`pi-tools.json`](../contracts/spec141/generated-capability-v2/pi-tools.json). Human per-tool references: [`docs/focusa-tools/tools/`](../focusa-tools/tools/).

## Registered routes

### `/connect`

- Methods: `GET /connect`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/connect/firstrun`

- Methods: `GET /connect/firstrun`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/connect/room/{room_id}/scan`

- Methods: `GET /connect/room/{room_id}/scan`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/connect/{room_id}`

- Methods: `GET /connect/{room_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/llms.txt`

- Methods: `GET /llms.txt`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/llms_txt.rs`
- Agent operations: none

### `/mcp`

- Methods: `POST /mcp`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mcp.rs`
- Agent operations: none

### `/pair/{device_id}`

- Methods: `GET /pair/{device_id}`
- Classification: `public_pairing`
- Rationale: Pairing/auth/license bootstrap surface; governed by its own token and expiry checks.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/pair/{device_id}/manifest.json`

- Methods: `GET /pair/{device_id}/manifest.json`
- Classification: `public_pairing`
- Rationale: Pairing/auth/license bootstrap surface; governed by its own token and expiry checks.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/pair/{device_id}/sw.js`

- Methods: `GET /pair/{device_id}/sw.js`
- Classification: `public_pairing`
- Rationale: Pairing/auth/license bootstrap surface; governed by its own token and expiry checks.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/proxy/acp`

- Methods: `POST /proxy/acp`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/proxy.rs`
- Agent operations: none

### `/proxy/v1/chat/completions`

- Methods: `POST /proxy/v1/chat/completions`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/proxy.rs`
- Agent operations: none

### `/proxy/v1/messages`

- Methods: `POST /proxy/v1/messages`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/proxy.rs`
- Agent operations: none

### `/v1/about`

- Methods: `GET /v1/about`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/health.rs`
- Agent operations: none

### `/v1/activation/status`

- Methods: `GET /v1/activation/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/license.rs`
- Agent operations: none

### `/v1/agent-runtime/amendments/activate`

- Methods: `POST /v1/agent-runtime/amendments/activate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_integrity.rs`
- Agent operations: none

### `/v1/agent-runtime/amendments/propose`

- Methods: `POST /v1/agent-runtime/amendments/propose`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_integrity.rs`
- Agent operations: none

### `/v1/agent-runtime/compile/agents-md`

- Methods: `POST /v1/agent-runtime/compile/agents-md`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/compile/skills`

- Methods: `POST /v1/agent-runtime/compile/skills`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/compile/system-prompt`

- Methods: `POST /v1/agent-runtime/compile/system-prompt`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/compile/target`

- Methods: `POST /v1/agent-runtime/compile/target`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/constitutions/draft`

- Methods: `POST /v1/agent-runtime/constitutions/draft`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/constitutions/{id}`

- Methods: `GET /v1/agent-runtime/constitutions/{id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/constitutions/{id}/activate`

- Methods: `POST /v1/agent-runtime/constitutions/{id}/activate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/constitutions/{id}/approve`

- Methods: `POST /v1/agent-runtime/constitutions/{id}/approve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/constitutions/{id}/preview`

- Methods: `POST /v1/agent-runtime/constitutions/{id}/preview`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/constitutions/{id}/revoke`

- Methods: `POST /v1/agent-runtime/constitutions/{id}/revoke`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/constitutions/{id}/rollback`

- Methods: `POST /v1/agent-runtime/constitutions/{id}/rollback`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/delivery/commit`

- Methods: `POST /v1/agent-runtime/delivery/commit`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/delivery/preview`

- Methods: `POST /v1/agent-runtime/delivery/preview`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/delivery/status`

- Methods: `GET /v1/agent-runtime/delivery/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/delivery/verify`

- Methods: `POST /v1/agent-runtime/delivery/verify`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/doctor`

- Methods: `GET /v1/agent-runtime/doctor`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_delivery.rs`
- Agent operations: none

### `/v1/agent-runtime/evaluations`

- Methods: `POST /v1/agent-runtime/evaluations`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/evaluations/{id}`

- Methods: `GET /v1/agent-runtime/evaluations/{id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/headless/verify`

- Methods: `POST /v1/agent-runtime/headless/verify`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_integrity.rs`
- Agent operations: none

### `/v1/agent-runtime/instruction-integrity/evaluate`

- Methods: `POST /v1/agent-runtime/instruction-integrity/evaluate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_integrity.rs`
- Agent operations: none

### `/v1/agent-runtime/instruction-integrity/status`

- Methods: `GET /v1/agent-runtime/instruction-integrity/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_integrity.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/claims`

- Methods: `GET /v1/agent-runtime/instructions/claims`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/conflicts`

- Methods: `GET /v1/agent-runtime/instructions/conflicts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/drift`

- Methods: `GET /v1/agent-runtime/instructions/drift`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/effective`

- Methods: `GET /v1/agent-runtime/instructions/effective`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/reconcile`

- Methods: `POST /v1/agent-runtime/instructions/reconcile`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/scan`

- Methods: `POST /v1/agent-runtime/instructions/scan`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/simulate`

- Methods: `POST /v1/agent-runtime/instructions/simulate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/instructions/sources`

- Methods: `GET /v1/agent-runtime/instructions/sources`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent-runtime/migration/preview`

- Methods: `POST /v1/agent-runtime/migration/preview`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_migration.rs`
- Agent operations: none

### `/v1/agent-runtime/studio`

- Methods: `GET /v1/agent-runtime/studio`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime_studio.rs`
- Agent operations: none

### `/v1/agent-runtime/variants/{id}`

- Methods: `GET /v1/agent-runtime/variants/{id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_runtime.rs`
- Agent operations: none

### `/v1/agent/adapter-capabilities`

- Methods: `GET /v1/agent/adapter-capabilities`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/capabilities`

- Methods: `GET /v1/agent/capabilities`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/card`

- Methods: `GET /v1/agent/card`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/compatibility-lock`

- Methods: `GET /v1/agent/compatibility-lock`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: `focusa.compatibility_lock.read`

### `/v1/agent/handshake`

- Methods: `POST /v1/agent/handshake`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: `focusa.protocol.handshake`

### `/v1/agent/operations`

- Methods: `GET /v1/agent/operations`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: `focusa.operation_registry.read`

### `/v1/agent/prompt`

- Methods: `GET /v1/agent/prompt`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/agent_reminder.rs`
- Agent operations: none

### `/v1/agent/schemas`

- Methods: `GET /v1/agent/schemas`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/schemas/{schema_id}`

- Methods: `GET /v1/agent/schemas/{schema_id}`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/tool-bundles`

- Methods: `GET /v1/agent/tool-bundles`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/tool-changes`

- Methods: `GET /v1/agent/tool-changes`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/tool-graph`

- Methods: `GET /v1/agent/tool-graph`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/tools`

- Methods: `GET /v1/agent/tools`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/tools/{name}`

- Methods: `GET /v1/agent/tools/{name}`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/agent/ui-action-bindings`

- Methods: `GET /v1/agent/ui-action-bindings`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: `focusa.ui_action_bindings.read`

### `/v1/agent/ui-capabilities`

- Methods: `GET /v1/agent/ui-capabilities`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: `focusa.ui_capability_snapshot.read`

### `/v1/agents`

- Methods: `GET /v1/agents`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/agents/{agent_id}`

- Methods: `GET /v1/agents/{agent_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/agents/{agent_id}/capabilities`

- Methods: `GET /v1/agents/{agent_id}/capabilities`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/agents/{agent_id}/constitution`

- Methods: `GET /v1/agents/{agent_id}/constitution`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/ascc/frame/{frame_id}`

- Methods: `GET /v1/ascc/frame/{frame_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ascc.rs`
- Agent operations: none

### `/v1/ascc/state`

- Methods: `GET /v1/ascc/state`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ascc.rs`
- Agent operations: none

### `/v1/ascc/update-delta`

- Methods: `POST /v1/ascc/update-delta`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ascc.rs`
- Agent operations: none

### `/v1/attachments/attach`

- Methods: `POST /v1/attachments/attach`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/attachments.rs`
- Agent operations: none

### `/v1/attachments/detach`

- Methods: `POST /v1/attachments/detach`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/attachments.rs`
- Agent operations: none

### `/v1/attachments/list`

- Methods: `GET /v1/attachments/list`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/attachments.rs`
- Agent operations: none

### `/v1/autonomy`

- Methods: `GET /v1/autonomy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/autonomy.rs`
- Agent operations: none

### `/v1/autonomy/explain`

- Methods: `GET /v1/autonomy/explain`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/autonomy/history`

- Methods: `GET /v1/autonomy/history`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/autonomy.rs`
- Agent operations: none

### `/v1/autonomy/ledger`

- Methods: `GET /v1/autonomy/ledger`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/autonomy/status`

- Methods: `GET /v1/autonomy/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/awareness/card`

- Methods: `GET /v1/awareness/card`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/awareness.rs`
- Agent operations: none

### `/v1/awareness/packet`

- Methods: `GET /v1/awareness/packet`, `POST /v1/awareness/packet`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/utility.rs`
- Agent operations: `focusa.awareness.packet`

### `/v1/awareness/packet/{surface}`

- Methods: `GET /v1/awareness/packet/{surface}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/utility.rs`
- Agent operations: none

### `/v1/bloatgaurd/domain/{name}`

- Methods: `GET /v1/bloatgaurd/domain/{name}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/gate-modes/mode/{name}`

- Methods: `GET /v1/bloatgaurd/gate-modes/mode/{name}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/gate-modes/report`

- Methods: `GET /v1/bloatgaurd/gate-modes/report`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/optical/imaged-kinds`

- Methods: `GET /v1/bloatgaurd/optical/imaged-kinds`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd_optical.rs`
- Agent operations: none

### `/v1/bloatgaurd/optical/ledger`

- Methods: `GET /v1/bloatgaurd/optical/ledger`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd_optical.rs`
- Agent operations: none

### `/v1/bloatgaurd/optical/ledger/{provider}`

- Methods: `GET /v1/bloatgaurd/optical/ledger/{provider}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd_optical.rs`
- Agent operations: none

### `/v1/bloatgaurd/optical/never-imaged`

- Methods: `GET /v1/bloatgaurd/optical/never-imaged`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd_optical.rs`
- Agent operations: none

### `/v1/bloatgaurd/optical/policy`

- Methods: `GET /v1/bloatgaurd/optical/policy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd_optical.rs`
- Agent operations: none

### `/v1/bloatgaurd/optical/probe`

- Methods: `GET /v1/bloatgaurd/optical/probe`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd_optical.rs`
- Agent operations: none

### `/v1/bloatgaurd/profiles/profile/{name}`

- Methods: `GET /v1/bloatgaurd/profiles/profile/{name}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/profiles/report`

- Methods: `GET /v1/bloatgaurd/profiles/report`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/report`

- Methods: `GET /v1/bloatgaurd/report`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: `focusa.bloatgaurd.report`

### `/v1/bloatgaurd/rollout/report`

- Methods: `GET /v1/bloatgaurd/rollout/report`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/routines/report`

- Methods: `GET /v1/bloatgaurd/routines/report`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/routines/routine/{name}`

- Methods: `GET /v1/bloatgaurd/routines/routine/{name}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/tokenbloat/domain/{name}`

- Methods: `GET /v1/bloatgaurd/tokenbloat/domain/{name}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/bloatgaurd/tokenbloat/report`

- Methods: `GET /v1/bloatgaurd/tokenbloat/report`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/bloatgaurd.rs`
- Agent operations: none

### `/v1/browser/capabilities/intake`

- Methods: `POST /v1/browser/capabilities/intake`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/browser_interop.rs`
- Agent operations: none

### `/v1/browser/webmcp/intake`

- Methods: `POST /v1/browser/webmcp/intake`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/browser_interop.rs`
- Agent operations: none

### `/v1/browser/workflow/plan`

- Methods: `POST /v1/browser/workflow/plan`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/browser_interop.rs`
- Agent operations: none

### `/v1/cache/events`

- Methods: `GET /v1/cache/events`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/cache/policy`

- Methods: `GET /v1/cache/policy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/cache/status`

- Methods: `GET /v1/cache/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/call-stack/design`

- Methods: `POST /v1/call-stack/design`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/call_stack.rs`
- Agent operations: `focusa.call_stack.design`

### `/v1/call-stack/list`

- Methods: `GET /v1/call-stack/list`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/call_stack.rs`
- Agent operations: none

### `/v1/call-stack/show`

- Methods: `GET /v1/call-stack/show`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/call_stack.rs`
- Agent operations: none

### `/v1/call-stack/verify`

- Methods: `POST /v1/call-stack/verify`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/call_stack.rs`
- Agent operations: `focusa.call_stack.verify`

### `/v1/clt/nodes`

- Methods: `GET /v1/clt/nodes`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/clt.rs`
- Agent operations: none

### `/v1/clt/path`

- Methods: `GET /v1/clt/path`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/clt.rs`
- Agent operations: none

### `/v1/clt/stats`

- Methods: `GET /v1/clt/stats`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/clt.rs`
- Agent operations: none

### `/v1/commands/log/{command_id}`

- Methods: `GET /v1/commands/log/{command_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/commands.rs`
- Agent operations: none

### `/v1/commands/status/{command_id}`

- Methods: `GET /v1/commands/status/{command_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/commands.rs`
- Agent operations: none

### `/v1/commands/submit`

- Methods: `POST /v1/commands/submit`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/commands.rs`
- Agent operations: none

### `/v1/compaction/build`

- Methods: `POST /v1/compaction/build`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/compaction/diff`

- Methods: `POST /v1/compaction/diff`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/compaction/evaluate`

- Methods: `POST /v1/compaction/evaluate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/compaction/inspect/{packet_id}`

- Methods: `GET /v1/compaction/inspect/{packet_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/compaction/packet/{packet_id}`

- Methods: `GET /v1/compaction/packet/{packet_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/compaction/policy`

- Methods: `GET /v1/compaction/policy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy.rs`
- Agent operations: none

### `/v1/compaction/policy/canary/enroll`

- Methods: `POST /v1/compaction/policy/canary/enroll`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/policy/canary/pause`

- Methods: `POST /v1/compaction/policy/canary/pause`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/policy/candidates`

- Methods: `GET /v1/compaction/policy/candidates`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/policy/evidence`

- Methods: `GET /v1/compaction/policy/evidence`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/policy/observe`

- Methods: `POST /v1/compaction/policy/observe`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/policy/override`

- Methods: `POST /v1/compaction/policy/override`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy.rs`
- Agent operations: none

### `/v1/compaction/policy/report`

- Methods: `POST /v1/compaction/policy/report`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy.rs`
- Agent operations: none

### `/v1/compaction/policy/resolve`

- Methods: `POST /v1/compaction/policy/resolve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/policy/rollback`

- Methods: `POST /v1/compaction/policy/rollback`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/policy/status`

- Methods: `GET /v1/compaction/policy/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction_policy_resolution.rs`
- Agent operations: none

### `/v1/compaction/prepare`

- Methods: `POST /v1/compaction/prepare`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/compaction/replay`

- Methods: `POST /v1/compaction/replay`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/compaction/verify`

- Methods: `POST /v1/compaction/verify`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/compaction.rs`
- Agent operations: none

### `/v1/connect/approve`

- Methods: `POST /v1/connect/approve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/room/create`

- Methods: `POST /v1/connect/room/create`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/room/firstrun`

- Methods: `POST /v1/connect/room/firstrun`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/room/start`

- Methods: `POST /v1/connect/room/start`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/room/{room_id}/approve`

- Methods: `POST /v1/connect/room/{room_id}/approve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/room/{room_id}/join`

- Methods: `POST /v1/connect/room/{room_id}/join`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/room/{room_id}/mac-offer`

- Methods: `POST /v1/connect/room/{room_id}/mac-offer`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/room/{room_id}/status`

- Methods: `GET /v1/connect/room/{room_id}/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/rooms`

- Methods: `GET /v1/connect/rooms`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/start`

- Methods: `POST /v1/connect/start`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/connect/status`

- Methods: `GET /v1/connect/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/constitution`

- Methods: `POST /v1/constitution`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/constitution.rs`
- Agent operations: none

### `/v1/constitution/active`

- Methods: `GET /v1/constitution/active`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/constitution.rs`
- Agent operations: none

### `/v1/constitution/diff`

- Methods: `GET /v1/constitution/diff`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/constitution/drafts`

- Methods: `GET /v1/constitution/drafts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/constitution/load`

- Methods: `POST /v1/constitution/load`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/constitution.rs`
- Agent operations: none

### `/v1/constitution/propose`

- Methods: `POST /v1/constitution/propose`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/constitution/versions`

- Methods: `GET /v1/constitution/versions`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/constitution.rs`
- Agent operations: none

### `/v1/context-cognition`

- Methods: `GET /v1/context-cognition`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: none

### `/v1/context-cognition/curate`

- Methods: `POST /v1/context-cognition/curate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: `focusa.context_cognition.curate`

### `/v1/context-cognition/curate/eval`

- Methods: `POST /v1/context-cognition/curate/eval`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: none

### `/v1/context-cognition/curate/eval/runs`

- Methods: `GET /v1/context-cognition/curate/eval/runs`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: none

### `/v1/context-cognition/curate/optimize`

- Methods: `POST /v1/context-cognition/curate/optimize`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: none

### `/v1/context-cognition/optimizer/artifacts`

- Methods: `GET /v1/context-cognition/optimizer/artifacts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: none

### `/v1/context-cognition/proof`

- Methods: `GET /v1/context-cognition/proof`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: none

### `/v1/context-cognition/render`

- Methods: `GET /v1/context-cognition/render`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/context_cognition.rs`
- Agent operations: none

### `/v1/context/adapters/docling/health`

- Methods: `GET /v1/context/adapters/docling/health`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_sources.rs`
- Agent operations: `focusa.context.adapter.docling.health`

### `/v1/context/graph`

- Methods: `GET /v1/context/graph`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_claims.rs`
- Agent operations: `focusa.context.graph.read`

### `/v1/context/graph/mutate`

- Methods: `POST /v1/context/graph/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_claims.rs`
- Agent operations: `focusa.context.graph.mutate`

### `/v1/context/retrieve`

- Methods: `POST /v1/context/retrieve`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_sources.rs`
- Agent operations: `focusa.context.retrieve`

### `/v1/context/sources`

- Methods: `GET /v1/context/sources`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_sources.rs`
- Agent operations: `focusa.context.source.list`

### `/v1/context/sources/commit`

- Methods: `POST /v1/context/sources/commit`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_sources.rs`
- Agent operations: `focusa.context.source.commit`

### `/v1/context/sources/ingest`

- Methods: `POST /v1/context/sources/ingest`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/context_sources.rs`
- Agent operations: `focusa.context.source.ingest`

### `/v1/contribute/approve`

- Methods: `POST /v1/contribute/approve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/training.rs`
- Agent operations: none

### `/v1/contribute/enable`

- Methods: `POST /v1/contribute/enable`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/training.rs`
- Agent operations: none

### `/v1/contribute/pause`

- Methods: `POST /v1/contribute/pause`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/training.rs`
- Agent operations: none

### `/v1/contribute/policy`

- Methods: `GET /v1/contribute/policy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/contribute/queue`

- Methods: `GET /v1/contribute/queue`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/contribute/status`

- Methods: `GET /v1/contribute/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/contribute/submit`

- Methods: `POST /v1/contribute/submit`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/training.rs`
- Agent operations: none

### `/v1/daemon-routing/resolve`

- Methods: `POST /v1/daemon-routing/resolve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/daemon_routing.rs`
- Agent operations: none

### `/v1/debug/set-pressure-threshold`

- Methods: `GET /v1/debug/set-pressure-threshold`
- Classification: `internal`
- Rationale: Runtime/operator diagnostic surface not projected as an agent capability.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/deck/home`

- Methods: `GET /v1/deck/home`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/deck.rs`
- Agent operations: none

### `/v1/deck/next-safe-action`

- Methods: `GET /v1/deck/next-safe-action`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/deck.rs`
- Agent operations: none

### `/v1/deck/proof-meter`

- Methods: `GET /v1/deck/proof-meter`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/deck.rs`
- Agent operations: none

### `/v1/deck/recall/schema`

- Methods: `GET /v1/deck/recall/schema`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/deck.rs`
- Agent operations: none

### `/v1/deck/walkthroughs`

- Methods: `GET /v1/deck/walkthroughs`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/deck.rs`
- Agent operations: none

### `/v1/device/pair/complete`

- Methods: `POST /v1/device/pair/complete`
- Classification: `public_pairing`
- Rationale: Pairing/auth/license bootstrap surface; governed by its own token and expiry checks.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/device/pair/list`

- Methods: `GET /v1/device/pair/list`
- Classification: `public_pairing`
- Rationale: Pairing/auth/license bootstrap surface; governed by its own token and expiry checks.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/device/pair/revoke`

- Methods: `POST /v1/device/pair/revoke`
- Classification: `public_pairing`
- Rationale: Pairing/auth/license bootstrap surface; governed by its own token and expiry checks.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: none

### `/v1/device/pair/start`

- Methods: `POST /v1/device/pair/start`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: `focusa.device_pair.start`

### `/v1/device/pair/status`

- Methods: `GET /v1/device/pair/status`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/device_pairing.rs`
- Agent operations: `focusa.device_pair.status`

### `/v1/doctor`

- Methods: `GET /v1/doctor`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/health.rs`
- Agent operations: none

### `/v1/doctor/closure`

- Methods: `GET /v1/doctor/closure`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_items.rs`
- Agent operations: none

### `/v1/dxux/digest`

- Methods: `GET /v1/dxux/digest`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/dxux.rs`
- Agent operations: none

### `/v1/dxux/explain/{failure}`

- Methods: `GET /v1/dxux/explain/{failure}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/dxux.rs`
- Agent operations: none

### `/v1/dxux/report`

- Methods: `GET /v1/dxux/report`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/dxux.rs`
- Agent operations: `focusa.dxux.report`

### `/v1/dxux/requirement/{id}`

- Methods: `GET /v1/dxux/requirement/{id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/dxux.rs`
- Agent operations: none

### `/v1/ecs/content/{handle_id}`

- Methods: `GET /v1/ecs/content/{handle_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ecs.rs`
- Agent operations: none

### `/v1/ecs/handles`

- Methods: `GET /v1/ecs/handles`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ecs.rs`
- Agent operations: none

### `/v1/ecs/rehydrate/{handle_id}`

- Methods: `POST /v1/ecs/rehydrate/{handle_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ecs.rs`
- Agent operations: none

### `/v1/ecs/resolve/{handle_id}`

- Methods: `GET /v1/ecs/resolve/{handle_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ecs.rs`
- Agent operations: none

### `/v1/ecs/store`

- Methods: `POST /v1/ecs/store`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ecs.rs`
- Agent operations: none

### `/v1/env`

- Methods: `GET /v1/env`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/env.rs`
- Agent operations: none

### `/v1/events/health`

- Methods: `GET /v1/events/health`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sse.rs`
- Agent operations: none

### `/v1/events/recall-trigger`

- Methods: `POST /v1/events/recall-trigger`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/turn_recent.rs`
- Agent operations: none

### `/v1/events/recent`

- Methods: `GET /v1/events/recent`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/events.rs`, `crates/focusa-api/src/routes/events_sqlite.rs`
- Agent operations: none

### `/v1/events/stream`

- Methods: `GET /v1/events/stream`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/events.rs`, `crates/focusa-api/src/routes/sse.rs`
- Agent operations: `focusa.events.stream`

### `/v1/events/{event_id}`

- Methods: `GET /v1/events/{event_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/events.rs`, `crates/focusa-api/src/routes/events_sqlite.rs`
- Agent operations: none

### `/v1/export/history`

- Methods: `GET /v1/export/history`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/export/manifest/{export_id}`

- Methods: `GET /v1/export/manifest/{export_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/export/run`

- Methods: `POST /v1/export/run`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/training.rs`
- Agent operations: none

### `/v1/export/status`

- Methods: `GET /v1/export/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/training.rs`
- Agent operations: none

### `/v1/focus-gate/candidates`

- Methods: `GET /v1/focus-gate/candidates`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/gate.rs`
- Agent operations: none

### `/v1/focus-gate/ingest-signal`

- Methods: `POST /v1/focus-gate/ingest-signal`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/gate.rs`
- Agent operations: none

### `/v1/focus-gate/pin`

- Methods: `POST /v1/focus-gate/pin`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/gate.rs`
- Agent operations: none

### `/v1/focus-gate/suppress`

- Methods: `POST /v1/focus-gate/suppress`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/gate.rs`
- Agent operations: none

### `/v1/focus-gate/surface`

- Methods: `POST /v1/focus-gate/surface`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/gate.rs`
- Agent operations: none

### `/v1/focus/frame/current`

- Methods: `GET /v1/focus/frame/current`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/focus.rs`
- Agent operations: none

### `/v1/focus/pop`

- Methods: `POST /v1/focus/pop`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/focus.rs`
- Agent operations: none

### `/v1/focus/push`

- Methods: `POST /v1/focus/push`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/focus.rs`
- Agent operations: none

### `/v1/focus/set-active`

- Methods: `POST /v1/focus/set-active`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/focus.rs`
- Agent operations: none

### `/v1/focus/snapshots`

- Methods: `POST /v1/focus/snapshots`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/snapshots.rs`
- Agent operations: none

### `/v1/focus/snapshots/diff`

- Methods: `POST /v1/focus/snapshots/diff`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/snapshots.rs`
- Agent operations: none

### `/v1/focus/snapshots/recent`

- Methods: `GET /v1/focus/snapshots/recent`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/snapshots.rs`
- Agent operations: none

### `/v1/focus/snapshots/restore`

- Methods: `POST /v1/focus/snapshots/restore`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/snapshots.rs`
- Agent operations: none

### `/v1/focus/stack`

- Methods: `GET /v1/focus/stack`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/focus.rs`
- Agent operations: none

### `/v1/focus/update`

- Methods: `POST /v1/focus/update`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/focus.rs`
- Agent operations: none

### `/v1/focusa/enabled`

- Methods: `GET /v1/focusa/enabled`, `PATCH /v1/focusa/enabled`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/focus.rs`
- Agent operations: none

### `/v1/gate/explain`

- Methods: `GET /v1/gate/explain`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/gate/policy`

- Methods: `GET /v1/gate/policy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/gate/scores`

- Methods: `GET /v1/gate/scores`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/gate/signal`

- Methods: `POST /v1/gate/signal`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/gate.rs`
- Agent operations: none

### `/v1/harnesses`

- Methods: `GET /v1/harnesses`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- Agent operations: none

### `/v1/harnesses/{harness}/capabilities`

- Methods: `GET /v1/harnesses/{harness}/capabilities`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- Agent operations: none

### `/v1/harnesses/{harness}/preflight`

- Methods: `POST /v1/harnesses/{harness}/preflight`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- Agent operations: none

### `/v1/health`

- Methods: `GET /v1/health`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/health.rs`
- Agent operations: `focusa.health.check`

### `/v1/hlt/history`

- Methods: `GET /v1/hlt/history`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: none

### `/v1/info`

- Methods: `GET /v1/info`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/info.rs`
- Agent operations: none

### `/v1/installations/convergence/plan`

- Methods: `POST /v1/installations/convergence/plan`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/convergence.rs`
- Agent operations: none

### `/v1/instances/connect`

- Methods: `POST /v1/instances/connect`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/instances.rs`
- Agent operations: none

### `/v1/instances/disconnect`

- Methods: `POST /v1/instances/disconnect`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/instances.rs`
- Agent operations: none

### `/v1/instances/list`

- Methods: `GET /v1/instances/list`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/instances.rs`
- Agent operations: none

### `/v1/interview/strategy/grill-with-docs/next-question`

- Methods: `POST /v1/interview/strategy/grill-with-docs/next-question`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/interview_strategy.rs`
- Agent operations: `focusa.interview.strategy.grill_with_docs.next_question`

### `/v1/interviews/closure-package`

- Methods: `GET /v1/interviews/closure-package`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/interview_sessions.rs`
- Agent operations: `focusa.interview.closure_package.get`

### `/v1/interviews/sessions`

- Methods: `GET /v1/interviews/sessions`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/interview_sessions.rs`
- Agent operations: `focusa.interview.session.list`

### `/v1/interviews/sessions/mutate`

- Methods: `POST /v1/interviews/sessions/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/interview_sessions.rs`
- Agent operations: `focusa.interview.session.mutate`

### `/v1/intuition/patterns`

- Methods: `GET /v1/intuition/patterns`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/intuition/signals`

- Methods: `GET /v1/intuition/signals`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/intuition/submit`

- Methods: `POST /v1/intuition/submit`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/license/status`

- Methods: `GET /v1/license/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/license.rs`
- Agent operations: none

### `/v1/lineage/children/{clt_node_id}`

- Methods: `GET /v1/lineage/children/{clt_node_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/lineage/head`

- Methods: `GET /v1/lineage/head`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: `focusa.lineage.head`

### `/v1/lineage/neighborhood/{clt_node_id}`

- Methods: `GET /v1/lineage/neighborhood/{clt_node_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/lineage/node/{clt_node_id}`

- Methods: `GET /v1/lineage/node/{clt_node_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/lineage/path/{clt_node_id}`

- Methods: `GET /v1/lineage/path/{clt_node_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/lineage/summaries`

- Methods: `GET /v1/lineage/summaries`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/lineage/tree`

- Methods: `GET /v1/lineage/tree`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: `focusa.lineage.tree`

### `/v1/mcp`

- Methods: `POST /v1/mcp`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mcp.rs`
- Agent operations: none

### `/v1/memory/procedural`

- Methods: `GET /v1/memory/procedural`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/memory.rs`
- Agent operations: `focusa.memory.procedural.read`

### `/v1/memory/procedural/reinforce`

- Methods: `POST /v1/memory/procedural/reinforce`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/memory.rs`
- Agent operations: `focusa.memory.procedural.reinforce`

### `/v1/memory/semantic`

- Methods: `GET /v1/memory/semantic`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/memory.rs`
- Agent operations: `focusa.memory.semantic.read`

### `/v1/memory/semantic/upsert`

- Methods: `POST /v1/memory/semantic/upsert`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/memory.rs`
- Agent operations: `focusa.memory.semantic.upsert`

### `/v1/metacognition/adjust`

- Methods: `POST /v1/metacognition/adjust`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: none

### `/v1/metacognition/adjustments/recent`

- Methods: `GET /v1/metacognition/adjustments/recent`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: none

### `/v1/metacognition/capture`

- Methods: `POST /v1/metacognition/capture`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: `focusa.metacog.capture`

### `/v1/metacognition/captures/{capture_id}`

- Methods: `GET /v1/metacognition/captures/{capture_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: none

### `/v1/metacognition/evaluate`

- Methods: `POST /v1/metacognition/evaluate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: none

### `/v1/metacognition/evaluations/recent`

- Methods: `GET /v1/metacognition/evaluations/recent`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: none

### `/v1/metacognition/reflect`

- Methods: `POST /v1/metacognition/reflect`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: `focusa.metacog.reflect`

### `/v1/metacognition/reflections/recent`

- Methods: `GET /v1/metacognition/reflections/recent`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: none

### `/v1/metacognition/retrieve`

- Methods: `POST /v1/metacognition/retrieve`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: `focusa.metacog.retrieve`

### `/v1/metacognition/status`

- Methods: `GET /v1/metacognition/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/metacognition.rs`
- Agent operations: none

### `/v1/metrics/perf`

- Methods: `GET /v1/metrics/perf`
- Classification: `internal`
- Rationale: Runtime/operator diagnostic surface not projected as an agent capability.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/metrics/session/{session_id}`

- Methods: `GET /v1/metrics/session/{session_id}`
- Classification: `internal`
- Rationale: Runtime/operator diagnostic surface not projected as an agent capability.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/metrics/ufi`

- Methods: `GET /v1/metrics/ufi`
- Classification: `internal`
- Rationale: Runtime/operator diagnostic surface not projected as an agent capability.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/metrics/uxp`

- Methods: `GET /v1/metrics/uxp`
- Classification: `internal`
- Rationale: Runtime/operator diagnostic surface not projected as an agent capability.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/mission-canvas/state`

- Methods: `GET /v1/mission-canvas/state`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- Agent operations: `focusa.mission_canvas.state.get`

### `/v1/mission-canvas/state/mutate`

- Methods: `POST /v1/mission-canvas/state/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- Agent operations: `focusa.mission_canvas.state.mutate`

### `/v1/mission-canvas/surface-bindings`

- Methods: `GET /v1/mission-canvas/surface-bindings`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- Agent operations: `focusa.mission_canvas.surface_binding.list`

### `/v1/mission-canvas/surface-bindings/mutate`

- Methods: `POST /v1/mission-canvas/surface-bindings/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- Agent operations: `focusa.mission_canvas.surface_binding.mutate`

### `/v1/mission-canvas/surfaces`

- Methods: `GET /v1/mission-canvas/surfaces`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- Agent operations: `focusa.mission_canvas.surface.list`

### `/v1/mission-canvas/surfaces/mutate`

- Methods: `POST /v1/mission-canvas/surfaces/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/mission_canvas_surfaces.rs`
- Agent operations: `focusa.mission_canvas.surface.mutate`

### `/v1/ontology/actions`

- Methods: `POST /v1/ontology/actions`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/adjacency`

- Methods: `GET /v1/ontology/adjacency`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/affordances`

- Methods: `GET /v1/ontology/affordances`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/communities`

- Methods: `GET /v1/ontology/communities`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/context`

- Methods: `POST /v1/ontology/context`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/contracts`

- Methods: `GET /v1/ontology/contracts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/domain-pack`

- Methods: `GET /v1/ontology/domain-pack`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/execution-critic`

- Methods: `POST /v1/ontology/execution-critic`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/intelligence-dashboard`

- Methods: `GET /v1/ontology/intelligence-dashboard`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/memory-pipeline`

- Methods: `POST /v1/ontology/memory-pipeline`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/primitives`

- Methods: `GET /v1/ontology/primitives`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/reflection-synthesizer`

- Methods: `POST /v1/ontology/reflection-synthesizer`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/retrieval-governor`

- Methods: `POST /v1/ontology/retrieval-governor`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/scope-migrations`

- Methods: `POST /v1/ontology/scope-migrations`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/slices`

- Methods: `GET /v1/ontology/slices`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/tool-choreography`

- Methods: `GET /v1/ontology/tool-choreography`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/tool-contracts`

- Methods: `GET /v1/ontology/tool-contracts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/tool-result-proposals`

- Methods: `POST /v1/ontology/tool-result-proposals`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/working-set`

- Methods: `GET /v1/ontology/working-set`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/ontology/world`

- Methods: `GET /v1/ontology/world`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/ontology.rs`
- Agent operations: none

### `/v1/openapi.json`

- Methods: `GET /v1/openapi.json`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/agent_capabilities.rs`
- Agent operations: none

### `/v1/prediction-authority/events`

- Methods: `POST /v1/prediction-authority/events`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/prediction_authority.rs`
- Agent operations: `focusa.prediction_authority.append`

### `/v1/prediction-authority/projection`

- Methods: `GET /v1/prediction-authority/projection`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/prediction_authority.rs`
- Agent operations: `focusa.prediction_authority.projection`

### `/v1/predictions`

- Methods: `POST /v1/predictions`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/predictions.rs`
- Agent operations: none

### `/v1/predictions/capture-outcome`

- Methods: `POST /v1/predictions/capture-outcome`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/predictions.rs`
- Agent operations: none

### `/v1/predictions/recent`

- Methods: `GET /v1/predictions/recent`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/predictions.rs`
- Agent operations: `focusa.prediction.recent`

### `/v1/predictions/stats`

- Methods: `GET /v1/predictions/stats`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/predictions.rs`
- Agent operations: none

### `/v1/predictions/{prediction_id}/evaluate`

- Methods: `POST /v1/predictions/{prediction_id}/evaluate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/predictions.rs`
- Agent operations: none

### `/v1/preload/build`

- Methods: `GET /v1/preload/build`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/preload/doctor`

- Methods: `GET /v1/preload/doctor`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/preload/profiles`

- Methods: `GET /v1/preload/profiles`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/preload/receipt-commit`

- Methods: `POST /v1/preload/receipt-commit`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/preload/receipt-preview`

- Methods: `GET /v1/preload/receipt-preview`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/preload/render`

- Methods: `GET /v1/preload/render`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/preload/verify`

- Methods: `GET /v1/preload/verify`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/preload/write`

- Methods: `POST /v1/preload/write`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/preload.rs`
- Agent operations: none

### `/v1/project/bind`

- Methods: `POST /v1/project/bind`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/bootstrap/apply`

- Methods: `POST /v1/project/bootstrap/apply`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_bootstrap.rs`
- Agent operations: `focusa.project.bootstrap.apply`

### `/v1/project/bootstrap/preview`

- Methods: `POST /v1/project/bootstrap/preview`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_bootstrap.rs`
- Agent operations: `focusa.project.bootstrap.preview`

### `/v1/project/bootstrap/repair`

- Methods: `POST /v1/project/bootstrap/repair`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_bootstrap.rs`
- Agent operations: `focusa.project.bootstrap.repair`

### `/v1/project/bootstrap/status`

- Methods: `GET /v1/project/bootstrap/status`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_bootstrap.rs`
- Agent operations: `focusa.project.bootstrap.status`

### `/v1/project/card`

- Methods: `GET /v1/project/card`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/card/outcome`

- Methods: `POST /v1/project/card/outcome`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/current`

- Methods: `GET /v1/project/current`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/discover`

- Methods: `GET /v1/project/discover`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/genesis/commit`

- Methods: `POST /v1/project/genesis/commit`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_genesis.rs`
- Agent operations: `focusa.project.genesis.commit`

### `/v1/project/genesis/resume`

- Methods: `POST /v1/project/genesis/resume`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_genesis.rs`
- Agent operations: `focusa.project.genesis.resume`

### `/v1/project/genesis/start`

- Methods: `POST /v1/project/genesis/start`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_genesis.rs`
- Agent operations: `focusa.project.genesis.start`

### `/v1/project/genesis/status`

- Methods: `GET /v1/project/genesis/status`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project_genesis.rs`
- Agent operations: `focusa.project.genesis.status`

### `/v1/project/identity`

- Methods: `GET /v1/project/identity`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: `focusa.project.identity`

### `/v1/project/list`

- Methods: `GET /v1/project/list`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/new`

- Methods: `POST /v1/project/new`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/remove`

- Methods: `POST /v1/project/remove`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/session-transfer`

- Methods: `POST /v1/project/session-transfer`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/settings`

- Methods: `GET /v1/project/settings`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/status`

- Methods: `GET /v1/project/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/switch`

- Methods: `POST /v1/project/switch`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/templates`

- Methods: `GET /v1/project/templates`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/trajectory-guard`

- Methods: `POST /v1/project/trajectory-guard`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/use`

- Methods: `POST /v1/project/use`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: none

### `/v1/project/verify`

- Methods: `GET /v1/project/verify`, `POST /v1/project/verify`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/project.rs`
- Agent operations: `focusa.project.verify`

### `/v1/prompt/assemble`

- Methods: `POST /v1/prompt/assemble`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/turn.rs`
- Agent operations: none

### `/v1/proposals`

- Methods: `GET /v1/proposals`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/proposals.rs`
- Agent operations: none

### `/v1/proposals/focus-frame`

- Methods: `POST /v1/proposals/focus-frame`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/proposals.rs`
- Agent operations: none

### `/v1/proposals/resolve`

- Methods: `POST /v1/proposals/resolve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/proposals.rs`
- Agent operations: none

### `/v1/providers`

- Methods: `GET /v1/providers`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- Agent operations: none

### `/v1/providers/conformance`

- Methods: `POST /v1/providers/conformance`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/provider_execution.rs`
- Agent operations: `focusa.provider.conformance.evaluate`

### `/v1/providers/contracts`

- Methods: `GET /v1/providers/contracts`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/provider_execution.rs`
- Agent operations: `focusa.provider.contract.list`

### `/v1/providers/{provider}/models`

- Methods: `GET /v1/providers/{provider}/models`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- Agent operations: none

### `/v1/providers/{provider}/models/preflight`

- Methods: `POST /v1/providers/{provider}/models/preflight`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- Agent operations: none

### `/v1/references`

- Methods: `GET /v1/references`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/references/salient`

- Methods: `GET /v1/references/salient`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/references/search`

- Methods: `GET /v1/references/search`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/references/trace`

- Methods: `GET /v1/references/trace`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/references/{ref_id}`

- Methods: `GET /v1/references/{ref_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/references/{ref_id}/meta`

- Methods: `GET /v1/references/{ref_id}/meta`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/reflect/history`

- Methods: `GET /v1/reflect/history`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/reflection.rs`
- Agent operations: none

### `/v1/reflect/run`

- Methods: `POST /v1/reflect/run`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/reflection.rs`
- Agent operations: none

### `/v1/reflect/scheduler`

- Methods: `GET /v1/reflect/scheduler`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/reflection.rs`
- Agent operations: none

### `/v1/reflect/scheduler/tick`

- Methods: `POST /v1/reflect/scheduler/tick`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/reflection.rs`
- Agent operations: none

### `/v1/reflect/status`

- Methods: `GET /v1/reflect/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/reflection.rs`
- Agent operations: none

### `/v1/reflex/primitives`

- Methods: `GET /v1/reflex/primitives`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/reflex.rs`
- Agent operations: none

### `/v1/release/proof/status`

- Methods: `GET /v1/release/proof/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/release.rs`
- Agent operations: none

### `/v1/resource/mode`

- Methods: `GET /v1/resource/mode`, `POST /v1/resource/mode`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/resource.rs`
- Agent operations: `focusa.resource_mode`

### `/v1/rfm`

- Methods: `GET /v1/rfm`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/rfm.rs`
- Agent operations: none

### `/v1/roles/profiles`

- Methods: `GET /v1/roles/profiles`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/role_profiles.rs`
- Agent operations: `focusa.role_profile.list`

### `/v1/roles/profiles/draft`

- Methods: `POST /v1/roles/profiles/draft`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/role_profiles.rs`
- Agent operations: `focusa.role_profile.draft`

### `/v1/roles/profiles/review`

- Methods: `POST /v1/roles/profiles/review`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/role_profiles.rs`
- Agent operations: `focusa.role_profile.review`

### `/v1/semantic-integrity/artifacts`

- Methods: `GET /v1/semantic-integrity/artifacts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/semantic_integrity.rs`
- Agent operations: none

### `/v1/semantic-integrity/artifacts/{artifact_id}`

- Methods: `GET /v1/semantic-integrity/artifacts/{artifact_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/semantic_integrity.rs`
- Agent operations: none

### `/v1/semantic-integrity/operations`

- Methods: `GET /v1/semantic-integrity/operations`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/semantic_integrity.rs`
- Agent operations: none

### `/v1/semantic-integrity/operations/{operation_id}`

- Methods: `POST /v1/semantic-integrity/operations/{operation_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/semantic_integrity.rs`
- Agent operations: none

### `/v1/semantic-integrity/status`

- Methods: `GET /v1/semantic-integrity/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/semantic_integrity.rs`
- Agent operations: none

### `/v1/session/bind`

- Methods: `POST /v1/session/bind`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/session/close`

- Methods: `POST /v1/session/close`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/session/discover`

- Methods: `GET /v1/session/discover`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/session/resume`

- Methods: `POST /v1/session/resume`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/session/start`

- Methods: `POST /v1/session/start`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/silent-sessions`

- Methods: `GET /v1/silent-sessions`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/capabilities`

- Methods: `GET /v1/silent-sessions/capabilities`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_capabilities.rs`
- Agent operations: none

### `/v1/silent-sessions/config/resolve`

- Methods: `POST /v1/silent-sessions/config/resolve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_config_read.rs`
- Agent operations: none

### `/v1/silent-sessions/preflight`

- Methods: `POST /v1/silent-sessions/preflight`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/presets`

- Methods: `GET /v1/silent-sessions/presets`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_config_read.rs`
- Agent operations: none

### `/v1/silent-sessions/profiles`

- Methods: `GET /v1/silent-sessions/profiles`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_config_read.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}`

- Methods: `GET /v1/silent-sessions/{session_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/adopt`

- Methods: `POST /v1/silent-sessions/{session_id}/adopt`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/artifacts`

- Methods: `GET /v1/silent-sessions/{session_id}/artifacts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_projection.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/cancel`

- Methods: `POST /v1/silent-sessions/{session_id}/cancel`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/checkpoints`

- Methods: `GET /v1/silent-sessions/{session_id}/checkpoints`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_projection.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/config/preview`

- Methods: `POST /v1/silent-sessions/{session_id}/config/preview`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_config_read.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/config/revisions`

- Methods: `POST /v1/silent-sessions/{session_id}/config/revisions`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_config_mutation.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/config/rollback`

- Methods: `POST /v1/silent-sessions/{session_id}/config/rollback`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_config_mutation.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/events`

- Methods: `GET /v1/silent-sessions/{session_id}/events`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/evidence-hold`

- Methods: `POST /v1/silent-sessions/{session_id}/evidence-hold`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_retention.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/export`

- Methods: `POST /v1/silent-sessions/{session_id}/export`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_retention.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/follow-up`

- Methods: `POST /v1/silent-sessions/{session_id}/follow-up`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_input.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/input`

- Methods: `POST /v1/silent-sessions/{session_id}/input`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_input.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/interrupt`

- Methods: `POST /v1/silent-sessions/{session_id}/interrupt`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/keys`

- Methods: `POST /v1/silent-sessions/{session_id}/keys`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_input.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/output`

- Methods: `GET /v1/silent-sessions/{session_id}/output`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/pause`

- Methods: `POST /v1/silent-sessions/{session_id}/pause`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/purge`

- Methods: `POST /v1/silent-sessions/{session_id}/purge`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_retention.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/receipts`

- Methods: `GET /v1/silent-sessions/{session_id}/receipts`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_projection.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/restart`

- Methods: `POST /v1/silent-sessions/{session_id}/restart`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/resume`

- Methods: `POST /v1/silent-sessions/{session_id}/resume`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/start`

- Methods: `POST /v1/silent-sessions/{session_id}/start`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/status`

- Methods: `GET /v1/silent-sessions/{session_id}/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/steer`

- Methods: `POST /v1/silent-sessions/{session_id}/steer`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_input.rs`
- Agent operations: none

### `/v1/silent-sessions/{session_id}/usage`

- Methods: `GET /v1/silent-sessions/{session_id}/usage`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/silent_sessions_projection.rs`
- Agent operations: none

### `/v1/skills`

- Methods: `GET /v1/skills`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/skills.rs`
- Agent operations: none

### `/v1/spec-workbench/session/mutate`

- Methods: `POST /v1/spec-workbench/session/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/spec_workbench.rs`
- Agent operations: `focusa.spec_workbench.session.mutate`

### `/v1/spec-workbench/sessions`

- Methods: `GET /v1/spec-workbench/sessions`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/spec_workbench.rs`
- Agent operations: `focusa.spec_workbench.session.list`

### `/v1/state/current`

- Methods: `GET /v1/state/current`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: `focusa.state.current`

### `/v1/state/diff`

- Methods: `GET /v1/state/diff`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/state/dump`

- Methods: `GET /v1/state/dump`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/state/explain`

- Methods: `GET /v1/state/explain`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/state/history`

- Methods: `GET /v1/state/history`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/state/stack`

- Methods: `GET /v1/state/stack`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities.rs`
- Agent operations: none

### `/v1/status`

- Methods: `GET /v1/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/status/deep`

- Methods: `GET /v1/status/deep`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/session.rs`
- Agent operations: none

### `/v1/subagent/result`

- Methods: `POST /v1/subagent/result`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/subagent.rs`
- Agent operations: none

### `/v1/sync/crdt/export`

- Methods: `GET /v1/sync/crdt/export`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/crdt/import`

- Methods: `POST /v1/sync/crdt/import`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/peers`

- Methods: `GET /v1/sync/peers`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/peers/{peer_id}`

- Methods: `DELETE /v1/sync/peers/{peer_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/pull/{peer_id}`

- Methods: `POST /v1/sync/pull/{peer_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/push/{peer_id}`

- Methods: `POST /v1/sync/push/{peer_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/receive`

- Methods: `POST /v1/sync/receive`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/status/{peer_id}`

- Methods: `GET /v1/sync/status/{peer_id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/sync/transfer`

- Methods: `POST /v1/sync/transfer`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/sync.rs`
- Agent operations: none

### `/v1/task-plans`

- Methods: `GET /v1/task-plans`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/task_plans.rs`
- Agent operations: `focusa.task_plan.list`

### `/v1/task-plans/materialize/beads`

- Methods: `POST /v1/task-plans/materialize/beads`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/task_plans.rs`
- Agent operations: `focusa.task_plan.materialize.beads`

### `/v1/task-plans/mutate`

- Methods: `POST /v1/task-plans/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/task_plans.rs`
- Agent operations: `focusa.task_plan.mutate`

### `/v1/telemetry/activity`

- Methods: `POST /v1/telemetry/activity`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/autonomy`

- Methods: `GET /v1/telemetry/autonomy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/cache-metadata`

- Methods: `POST /v1/telemetry/cache-metadata`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/cache-metadata/status`

- Methods: `GET /v1/telemetry/cache-metadata/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/cost`

- Methods: `GET /v1/telemetry/cost`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/event`

- Methods: `POST /v1/telemetry/event`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/events`

- Methods: `GET /v1/telemetry/events`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/memory`

- Methods: `GET /v1/telemetry/memory`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/ops`

- Methods: `POST /v1/telemetry/ops`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/process`

- Methods: `GET /v1/telemetry/process`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/telemetry/productivity`

- Methods: `GET /v1/telemetry/productivity`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/snapshot`

- Methods: `GET /v1/telemetry/snapshot`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/token-budget`

- Methods: `POST /v1/telemetry/token-budget`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/token-budget/status`

- Methods: `GET /v1/telemetry/token-budget/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/tokens`

- Methods: `GET /v1/telemetry/tokens`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/tool-usage`

- Methods: `POST /v1/telemetry/tool-usage`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/tools`

- Methods: `GET /v1/telemetry/tools`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/trace`

- Methods: `GET /v1/telemetry/trace`, `POST /v1/telemetry/trace`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/trace/batch`

- Methods: `POST /v1/telemetry/trace/batch`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/trace/stats`

- Methods: `GET /v1/telemetry/trace/stats`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/telemetry.rs`
- Agent operations: none

### `/v1/telemetry/ux`

- Methods: `GET /v1/telemetry/ux`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/capabilities_extra.rs`
- Agent operations: none

### `/v1/temporal/civil/resolve`

- Methods: `POST /v1/temporal/civil/resolve`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.civil.resolve`

### `/v1/temporal/clock/capture`

- Methods: `POST /v1/temporal/clock/capture`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.clock.capture`

### `/v1/temporal/commit`

- Methods: `POST /v1/temporal/commit`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.commit`

### `/v1/temporal/forecast`

- Methods: `POST /v1/temporal/forecast`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.forecast`

### `/v1/temporal/high-consequence/preflight`

- Methods: `POST /v1/temporal/high-consequence/preflight`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.high_consequence_preflight`

### `/v1/temporal/migrate-signatures`

- Methods: `POST /v1/temporal/migrate-signatures`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.migrate_signatures`

### `/v1/temporal/observe`

- Methods: `POST /v1/temporal/observe`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.observe`

### `/v1/temporal/preflight`

- Methods: `POST /v1/temporal/preflight`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.preflight`

### `/v1/temporal/priority/commit`

- Methods: `POST /v1/temporal/priority/commit`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.priority.commit`

### `/v1/temporal/revise`

- Methods: `POST /v1/temporal/revise`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.revise`

### `/v1/temporal/settle-closure`

- Methods: `POST /v1/temporal/settle-closure`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.settle_closure`

### `/v1/temporal/status`

- Methods: `GET /v1/temporal/status`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/temporal.rs`
- Agent operations: `focusa.temporal.status`

### `/v1/threads`

- Methods: `GET /v1/threads`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/threads.rs`
- Agent operations: none

### `/v1/threads/{id}`

- Methods: `GET /v1/threads/{id}`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/threads.rs`
- Agent operations: none

### `/v1/threads/{id}/fork`

- Methods: `POST /v1/threads/{id}/fork`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/threads.rs`
- Agent operations: none

### `/v1/threads/{id}/transfer`

- Methods: `POST /v1/threads/{id}/transfer`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/threads.rs`
- Agent operations: none

### `/v1/tokens/create`

- Methods: `POST /v1/tokens/create`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/tokens.rs`
- Agent operations: none

### `/v1/tokens/list`

- Methods: `GET /v1/tokens/list`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/tokens.rs`
- Agent operations: none

### `/v1/tokens/revoke`

- Methods: `POST /v1/tokens/revoke`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/tokens.rs`
- Agent operations: none

### `/v1/training/status`

- Methods: `GET /v1/training/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/training.rs`
- Agent operations: none

### `/v1/trajectory/assess`

- Methods: `POST /v1/trajectory/assess`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: `focusa.trajectory.assess`

### `/v1/trajectory/checkpoint`

- Methods: `POST /v1/trajectory/checkpoint`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: `focusa.trajectory.checkpoint`

### `/v1/trajectory/define-goal`

- Methods: `POST /v1/trajectory/define-goal`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: `focusa.trajectory.define_goal`

### `/v1/trajectory/history`

- Methods: `GET /v1/trajectory/history`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: none

### `/v1/trajectory/propose-workpoint`

- Methods: `POST /v1/trajectory/propose-workpoint`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: `focusa.trajectory.propose_workpoint`

### `/v1/trajectory/query`

- Methods: `GET /v1/trajectory/query`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: none

### `/v1/trajectory/resume`

- Methods: `POST /v1/trajectory/resume`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: `focusa.trajectory.resume`

### `/v1/trajectory/view`

- Methods: `GET /v1/trajectory/view`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/trajectory.rs`
- Agent operations: `focusa.trajectory.view`

### `/v1/traverse`

- Methods: `POST /v1/traverse`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/traverse.rs`
- Agent operations: `focusa.traverse`

### `/v1/traverse/verify-tags`

- Methods: `POST /v1/traverse/verify-tags`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/traverse.rs`
- Agent operations: none

### `/v1/trust/metrics`

- Methods: `PATCH /v1/trust/metrics`
- Classification: `internal`
- Rationale: Runtime/operator diagnostic surface not projected as an agent capability.
- Sources: `crates/focusa-api/src/routes/trust.rs`
- Agent operations: none

### `/v1/turn/append`

- Methods: `POST /v1/turn/append`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/turn.rs`
- Agent operations: `focusa.turn.append`

### `/v1/turn/complete`

- Methods: `POST /v1/turn/complete`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/turn.rs`
- Agent operations: `focusa.turn.complete`

### `/v1/turn/start`

- Methods: `POST /v1/turn/start`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/turn.rs`
- Agent operations: `focusa.turn.start`

### `/v1/turns/recent`

- Methods: `GET /v1/turns/recent`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/turn_recent.rs`
- Agent operations: none

### `/v1/ufi`

- Methods: `GET /v1/ufi`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/uxp.rs`
- Agent operations: none

### `/v1/update/admin`

- Methods: `POST /v1/update/admin`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/apply`

- Methods: `POST /v1/update/apply`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/check`

- Methods: `GET /v1/update/check`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/history`

- Methods: `GET /v1/update/history`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/notifications`

- Methods: `GET /v1/update/notifications`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/plan`

- Methods: `GET /v1/update/plan`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/policy`

- Methods: `GET /v1/update/policy`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/rollback`

- Methods: `POST /v1/update/rollback`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/scheduler`

- Methods: `GET /v1/update/scheduler`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/update/status`

- Methods: `GET /v1/update/status`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/update.rs`
- Agent operations: none

### `/v1/utility/bootstrap`

- Methods: `GET /v1/utility/bootstrap`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/utility.rs`
- Agent operations: none

### `/v1/utility/card`

- Methods: `GET /v1/utility/card`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/utility.rs`
- Agent operations: none

### `/v1/utility/post-compaction`

- Methods: `GET /v1/utility/post-compaction`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/utility.rs`
- Agent operations: none

### `/v1/uxp`

- Methods: `GET /v1/uxp`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/uxp.rs`
- Agent operations: none

### `/v1/visual-workflow/evidence`

- Methods: `GET /v1/visual-workflow/evidence`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/visual_workflow.rs`
- Agent operations: none

### `/v1/visual-workflow/evidence/store`

- Methods: `POST /v1/visual-workflow/evidence/store`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/visual_workflow.rs`
- Agent operations: none

### `/v1/work-items/closure/prepare`

- Methods: `POST /v1/work-items/closure/prepare`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_items.rs`
- Agent operations: none

### `/v1/work-items/closure/submit`

- Methods: `POST /v1/work-items/closure/submit`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_items.rs`
- Agent operations: none

### `/v1/work-items/closure/validate`

- Methods: `POST /v1/work-items/closure/validate`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_items.rs`
- Agent operations: none

### `/v1/work-items/providers`

- Methods: `GET /v1/work-items/providers`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_items.rs`
- Agent operations: none

### `/v1/work-loop`

- Methods: `GET /v1/work-loop`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/checkpoint`

- Methods: `POST /v1/work-loop/checkpoint`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/checkpoints`

- Methods: `GET /v1/work-loop/checkpoints`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/context`

- Methods: `POST /v1/work-loop/context`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/degraded`

- Methods: `POST /v1/work-loop/degraded`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/delegation/clear`

- Methods: `POST /v1/work-loop/delegation/clear`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/delegation/enable`

- Methods: `POST /v1/work-loop/delegation/enable`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/driver/abort`

- Methods: `POST /v1/work-loop/driver/abort`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: `focusa.agent_execution.abort`

### `/v1/work-loop/driver/prompt`

- Methods: `POST /v1/work-loop/driver/prompt`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: `focusa.agent_execution.prompt`

### `/v1/work-loop/driver/start`

- Methods: `POST /v1/work-loop/driver/start`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: `focusa.agent_execution.start`

### `/v1/work-loop/driver/stop`

- Methods: `POST /v1/work-loop/driver/stop`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: `focusa.agent_execution.stop`

### `/v1/work-loop/enable`

- Methods: `POST /v1/work-loop/enable`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/events`

- Methods: `POST /v1/work-loop/events`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/health`

- Methods: `GET /v1/work-loop/health`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/heartbeat`

- Methods: `POST /v1/work-loop/heartbeat`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/pause`

- Methods: `POST /v1/work-loop/pause`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/pause-flags`

- Methods: `POST /v1/work-loop/pause-flags`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/replay/closure-bundle`

- Methods: `GET /v1/work-loop/replay/closure-bundle`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/replay/closure-evidence`

- Methods: `GET /v1/work-loop/replay/closure-evidence`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/resume`

- Methods: `POST /v1/work-loop/resume`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/select-next`

- Methods: `POST /v1/work-loop/select-next`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/session/abort`

- Methods: `POST /v1/work-loop/session/abort`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/session/attach`

- Methods: `POST /v1/work-loop/session/attach`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/status`

- Methods: `GET /v1/work-loop/status`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: `focusa.work_loop.status`

### `/v1/work-loop/status/deep`

- Methods: `GET /v1/work-loop/status/deep`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-loop/stop`

- Methods: `POST /v1/work-loop/stop`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/work_loop.rs`
- Agent operations: none

### `/v1/work-rail`

- Methods: `GET /v1/work-rail`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/work_rail.rs`
- Agent operations: `focusa.work_rail.list`

### `/v1/work-rail/mutate`

- Methods: `POST /v1/work-rail/mutate`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/work_rail.rs`
- Agent operations: `focusa.work_rail.mutate`

### `/v1/workpoint/active-object/resolve`

- Methods: `POST /v1/workpoint/active-object/resolve`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: none

### `/v1/workpoint/checkpoint`

- Methods: `POST /v1/workpoint/checkpoint`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: `focusa.workpoint.checkpoint`

### `/v1/workpoint/current`

- Methods: `GET /v1/workpoint/current`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: none

### `/v1/workpoint/drift-check`

- Methods: `POST /v1/workpoint/drift-check`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: none

### `/v1/workpoint/evidence/link`

- Methods: `POST /v1/workpoint/evidence/link`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: none

### `/v1/workpoint/idempotency-cache`

- Methods: `GET /v1/workpoint/idempotency-cache`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: none

### `/v1/workpoint/resume`

- Methods: `GET /v1/workpoint/resume`, `POST /v1/workpoint/resume`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: `focusa.workpoint.resume`

### `/v1/workpoint/rollover/target-materialize`

- Methods: `POST /v1/workpoint/rollover/target-materialize`
- Classification: `operator_only`
- Rationale: Not in the curated agent operation registry; requires explicit operator/application workflow authority.
- Sources: `crates/focusa-api/src/routes/workpoint.rs`
- Agent operations: none

### `/v1/workspace/artifacts`

- Methods: `GET /v1/workspace/artifacts`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/workspace_artifacts.rs`
- Agent operations: `focusa.workspace.artifact.list`

### `/v1/workspace/artifacts/intake`

- Methods: `POST /v1/workspace/artifacts/intake`
- Classification: `agent_eligible`
- Rationale: Covered by the generated operation registry or Spec141 capability-discovery/MCP contract.
- Sources: `crates/focusa-api/src/routes/workspace_artifacts.rs`
- Agent operations: `focusa.workspace.artifact.intake`
