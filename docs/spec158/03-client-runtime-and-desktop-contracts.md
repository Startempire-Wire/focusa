# Spec 158 Companion 03 — Client, Runtime, Desktop, and Work Surface Contracts

**Status:** normative companion to Spec 158  
**Parent:** `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`

---

## 1. Client authority rule

CLI, Pi, menubar, Focusa Desktop, Focusa.work, MCP and external adapters are clients or projections. They do not own canonical cognition.

Every canonical-capable request SHALL carry or resolve an exact:

```text
ScopeRef
WorkstreamId
actor
capability/entitlement context
AttachmentKey when runtime binding matters
idempotency/causal metadata when mutation occurs
```

Clients may remember presentation preferences. They may not remember canonical current Workstream as mutation authority without re-resolution.

---

## 2. Workstream-aware API envelope

Canonical operations should converge on a shared envelope:

```json
{
  "schema": "focusa.workstream_operation_request.v1",
  "workstream": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_..."
  },
  "continuity_id": "cont_...",
  "attachment": null,
  "actor": {
    "actor_type": "operator|agent|pi|desktop|web|service",
    "actor_id": "..."
  },
  "command_id": "focusa.workpoint.resume",
  "input": {},
  "idempotency_key": "..."
}
```

The response echoes the resolved Workstream and returns a typed Receipt or recovery state.

---

## 3. Fail-closed resolution

Unbound or ambiguous requests return no foreign cognitive payload.

A bounded error may include:

- reason code;
- candidate stable Workstream refs;
- provenance explaining why candidates exist;
- exact repair or attachment action;
- no canonical mutation;
- no foreign Workpoint/Trajectory/Context content.

Compatibility endpoints may accept old identifiers only if the mapping is unique and explicit.

---

## 4. Pi runtime contract

The Pi extension SHALL add `workstream_id` to its typed runtime and Attachment keys.

Every event, tool, command, hook, shortcut, compaction path, model switch, recovery path and augmentation path SHALL resolve the same exact Workstream.

Required behavior:

- CWD is launch context, not authority;
- latest verified/last active fallback is not canonical;
- raw command stdout/stderr remains byte-faithful and separate from Focusa augmentation;
- augmentation includes provenance and exact Workstream verification;
- recovery sidecars include Workstream and workspace binding;
- compaction/resume packets include Scope fingerprint, WorkstreamId, Continuity generation, event head, Workpoint revision, Trajectory revision and source provenance;
- caches and adaptive state include Workstream/Attachment keys;
- model switch and module reload cannot carry another Workstream’s owner state.

---

## 5. Focusa Desktop control plane

Focusa Desktop is a semantic presentation and command surface.

Required route families:

```text
GET  /v1/desktop/manifest
GET  /v1/desktop/status
GET  /v1/desktop/state
GET  /v1/desktop/events
POST /v1/desktop/launch
POST /v1/desktop/present
POST /v1/desktop/navigate
POST /v1/desktop/layout/apply
POST /v1/desktop/surfaces/open
POST /v1/desktop/surfaces/focus
POST /v1/desktop/surfaces/close
POST /v1/desktop/surfaces/move
POST /v1/desktop/surfaces/split
POST /v1/desktop/surfaces/detach
POST /v1/desktop/commands/{command_id}/invoke
GET  /v1/desktop/operations/{operation_id}
```

A local socket/named pipe may optimize local delivery but must conform to the same typed contract.

### 5.1 Presentation request

```json
{
  "schema": "focusa.desktop_present_request.v1",
  "workspace_id": "mission_canvas",
  "subsection_id": "sessions",
  "view_id": null,
  "object_ref": "workpoint:wp_...",
  "work_surface_id": null,
  "window_mode": "focus_existing",
  "workstream": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_..."
  },
  "continuity_id": "cont_...",
  "requested_by": {
    "client_type": "pi|cli|agent|menubar|desktop|web",
    "client_id": "..."
  },
  "idempotency_key": "..."
}
```

### 5.2 Presentation receipt

```json
{
  "schema": "focusa.desktop_present_receipt.v1",
  "operation_id": "op_...",
  "status": "presented|already_present|blocked|unavailable|failed",
  "window_id": "window_main",
  "workspace_id": "mission_canvas",
  "subsection_id": "sessions",
  "object_ref": "workpoint:wp_...",
  "resolved_workstream": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_..."
  },
  "recovery": null,
  "receipt_ref": "receipt:..."
}
```

### 5.3 Semantic state

Desktop exposes bounded semantic state for agent verification:

```json
{
  "schema": "focusa.desktop_state.v1",
  "active_window_id": "window_main",
  "active_workspace_id": "mission_canvas",
  "active_subsection_id": "sessions",
  "active_object_ref": "session:...",
  "focused_work_surface_id": "surface_pi_01",
  "resolved_workstream": {
    "scope_ref": {"project_root_key": "prk_..."},
    "workstream_id": "ws_..."
  },
  "open_work_surfaces": [],
  "visible_commands": [],
  "pending_approvals": [],
  "blocked_states": [],
  "dialogs": [],
  "layout": {},
  "updated_at": "..."
}
```

Screenshot/image state is optional Evidence and diagnostics, not the primary control representation.

---

## 6. Work Surface contract

Every Work Surface SHALL contain:

```text
work_surface_id
kind
display_name
WorkstreamKey
optional ContinuityId
optional AttachmentKey
optional stable object_ref
renderer_id
window/layout group
pin/split/detach state
lifecycle/health/activity
approval/conflict/blocker counts
writer lease/worktree/browser isolation
presentation revision
```

Visual focus is not canonical authority.

Opening or selecting a Work Surface may request an explicit client Attachment switch. It may not silently alter daemon-global state.

---

## 7. GUI/CLI/agent parity

The following share one workspace and command registry:

- Desktop navigation;
- Desktop command palette;
- `focusa desktop` CLI;
- Pi tools/commands;
- Focusa agent tool registry;
- Focusa.work navigation/control.

For each command:

```text
stable command_id
human label
workspace/subsection placement
input/output schemas
owner route/service
scope requirements
capabilities
entitlement feature
side-effect class
approval requirements
idempotency behavior
Receipt behavior
recovery behavior
```

A Svelte-only click handler is not a complete product command.

---

## 8. CLI contract

Representative surface:

```text
focusa desktop manifest --json
focusa desktop status --json
focusa desktop state --json
focusa desktop present --project-root-key ... --workstream-id ... --workspace ...
focusa desktop events --jsonl
focusa desktop surface list --json
focusa desktop surface focus <id>
focusa desktop command list --json
focusa desktop command invoke <command_id> --input file.json
```

Rules:

- machine-complete JSON;
- stable exit codes;
- operation and Receipt refs;
- exact Workstream echo;
- no inferred mutation authority from local current selection;
- useful headless behavior when Desktop is closed.

---

## 9. Pi Work Surface

The interactive Pi Work Surface SHALL use a genuine PTY.

It must:

- retain the Pi process when hidden;
- propagate terminal resize;
- preserve authentic input/output;
- bind exact Workstream/Continuity/Attachment;
- support focus/split/detach/restore;
- expose stable WorkSurfaceId to CLI and agent;
- avoid silently terminating canonical work when the view closes.

Interactive Pi PTY and daemon-owned Pi RPC are separate processes/modes with distinct Attachment identities.

---

## 10. Menubar and Focusa.work

Menubar remains compact. It may present status and request a Desktop handoff; it does not own a second full Mission Canvas.

Focusa.work reuses stable workspace IDs, command IDs, object refs and Workstream identity through hosted, connected-local or self-hosted environment adapters.

---

## 11. UI and DTO restrictions

- generated OpenAPI types own transport DTOs;
- shared bounded projection packages own semantic transformations;
- clients do not independently guess `any` payloads;
- SvelteKit owns authored shell/composition;
- A2UI Lit owns generated surface rendering;
- Focusa Svelte Custom Elements own approved generated domain controls;
- no competing Svelte A2UI renderer;
- no Svelte store owns canonical cognition.

---

## 12. Client closure gates

- every canonical operation resolves exact Workstream;
- every Work Surface carries Workstream identity;
- GUI/CLI/agent parity is tested;
- visual focus cannot mutate authority;
- Desktop failure cannot corrupt Workstream state;
- Pi terminal fallback remains functional;
- menubar has no duplicate full Mission Canvas;
- Focusa.work uses the same semantic graph;
- no handwritten duplicate DTO/command registry remains.
