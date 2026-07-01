# Spec 115 — Focusa Cloud Control Plane, Tool Gateway, and Local-First SaaS Master Plan

## 0. Status

**Status:** proposed master specification
**Scope:** Focusa Cloud, hosted control plane, `npx focusa`, `ssh cloud.focusa.dev`, node registry, pairing, relay, MCP parity, tool gateway, code execution capsule, proof receipts, benchmark observatory, team/multi-node sync, multiplexing, pricing, launch plan, acceptance gates.
**Authority:** This spec extends existing Focusa docs and code surfaces. It does not redefine Workpoint, Focus State, Trajectory, Evidence, Context Authority, CRDT sync, device pairing, tool contracts, preload, benchmarks, or proof policies.

## 1. Normative basis

This spec depends on and preserves these existing Focusa surfaces:

| Existing surface                              | This spec uses it for                                           |
| --------------------------------------------- | --------------------------------------------------------------- |
| README / current runtime                      | local-first positioning, daemon/API/CLI/Pi/menubar architecture |
| Spec 43 — Multi-device sync                   | local-first peer sync, observation import, thread ownership     |
| Spec 53 — Device pairing                      | device trust, code/token pairing, connect pages                 |
| Spec 90 — Tool contracts                      | MCP parity, tool gateway generation, tool-family governance     |
| Spec 111 — Agent Context Bootstrap & Delivery | preload packets, agent readiness receipts                       |
| Spec 112 — Install Binary Architecture        | install, license, update, platform support                      |
| Spec 113 — Agent Performance Benchmark        | private and public benchmark evidence                           |
| Spec 114 — Public Benchmark Flywheel          | `bench.focusa.dev`, `evals.focusa.dev`, `proof.focusa.dev`      |
| Public stream / proof policies                | redaction, publish gates, proof receipts                        |
| Commercial/license docs                       | SaaS entitlement and commercial-use boundaries                  |

Focusa is already positioned as local-first, with state living on the machine running the daemon, and it explicitly is not a cloud memory service.  The current runtime snapshot already includes Rust daemon, HTTP API, CLI, TUI, Pi extension, and menubar proof in Operator Preview form.

## 2. One-line product definition

**Focusa Cloud is the hosted command center for self-hosted Focusa nodes.**

Short form:

```text
Self-host the work.
Cloud-coordinate the access.
Publish proof by consent.
Keep cognition local.
```

## 3. Master product promise

Focusa Cloud gives local-first developers cloud convenience without cloud custody.

The self-hosted Focusa node owns:

```text
Workpoints
Focus State
Trajectory
Context Authority
Evidence refs
ProjectIdentity
Context Cognition
local Eval Ledger
Pi tool execution
MCP tool execution
local project files
private diagnostics
private code context
```

Focusa Cloud owns:

```text
accounts
billing
licenses
node registration
device registration
SSH identity registration
cloud dashboard
tool entitlement
MCP client registry
optional secure relay
proof receipt hosting
benchmark snapshot hosting
support intake
team administration
```

Master rule:

```text
Cloud coordinates.
Node decides.
Receipts prove.
Private state stays local.
```

## 4. Product boundaries

### 4.1 Focusa Cloud is

```text
hosted control plane
license authority
node registry
device trust registry
tool entitlement layer
MCP client registry
SSH terminal cockpit
npx onboarding companion
relay coordinator
proof receipt publisher
benchmark observatory publisher
support workflow intake
team visibility layer
```

### 4.2 Focusa Cloud is not

```text
hosted Focus State
hosted Workpoint authority
cloud memory service
raw daemon proxy
generic ngrok clone
hosted project file store
raw prompt/log observability platform
cloud agent runtime
automatic cognitive merger
```

## 5. Existing codebase facts that anchor this plan

### 5.1 Current runtime shape

Focusa’s README describes an architecture where the Pi extension talks to the Rust daemon/API, which owns Focus State, Workpoints, reducer transitions, ontology, lineage, snapshots, metacognition, work-loop, ECS references, and CLI/API parity surfaces.

### 5.2 Tool surface exists and is contract-backed

The generated tool surface summary reports 97 tool contracts, 11 tool families, 93 API parity items, 81 CLI parity items, 96 Pi tools, and 97 docs coverage items.

Spec 90 requires every `focusa_*` Pi tool to have a canonical contract linking the tool to ontology actions, API routes, CLI parity, core/reducer surface, docs, result-envelope expectations, live health checks, and explicit exemptions.

Each tool contract includes fields such as `name`, `family`, `purpose`, `ontology_action`, `api_routes`, `cli_commands`, `core_surface`, `doc_path`, `result_envelope`, `side_effect_profile`, `parity_status`, exemptions, and live check.

### 5.3 Device pairing already exists

The Focusa API already exposes device pairing, connect sessions, connect rooms, `/connect`, `/pair/{device_id}`, manifest, and service worker routes.  The route implementation describes OAuth-like pairing where the Mac starts pairing, the daemon returns a code/device ID, the operator completes pairing, the daemon mints a token, and the Mac stores the token for later calls.

### 5.4 Sync and CRDT already exist

The sync API already exposes peer registry, push, pull, status, CRDT export/import, receive, and transfer routes.  The current multi-device sync doc requires bidirectional local-first sync, deterministic inspectable behavior, no silent cognitive merges, and future daisy-chain readiness.

Focusa’s CRDT module uses vector clocks, Lamport timestamps, event UUIDs, and machine IDs.  Remote CRDT merges ignore duplicate event IDs, append unseen events, merge vector clocks, advance Lamport counters, and sort events causally with Lamport fallback.

### 5.5 Multi-device authority already exists

Remote changes are imported as observations by default and must not directly mutate canonical Focus Stack or Focus State. Per-thread ownership makes the thread owner the canonical writer, while non-owners may observe, propose, or attach as assistant/observer.

The reducer enforces this: observations are recorded without mutating canonical state, and non-owner canonical mutations are rejected with an ownership violation.

### 5.6 Spec 111 adds agent preload

Spec 111 defines Focusa Agent Context Bootstrap & Delivery as a first-class layer that builds, renders, writes, and verifies bounded startup packets so new agent sessions can continue verified project work without relying on transcript tail or stale chat memory.  Its product promise is that before an AI coding agent acts, Focusa can prove which project, mission, Workpoint, next action, evidence refs, and drift boundaries it was given.

### 5.7 Spec 112 adds install/license authority

Spec 112 defines a smart installer that detects OS, architecture, libc, init system, dependencies, selects correct binaries, verifies artifacts, rolls back failures, integrates license validation, and provides AX-correct recovery hints.  It defines the license authority at `install.focusa.dev` with `/wp-json/wpuiai-ai-cloud/v1/license/validate`, plus license persistence and daemon revalidation behavior.

### 5.8 Spec 113 adds measured benchmark evidence

Spec 113 asks whether Specs 110, 111, 112, and future changes actually improve agent performance on real metrics over time.  It defines outcome, behavior, and experience metrics such as task completion, time, token efficiency, recovery, tool selection accuracy, drift, context overhead, license compliance, cross-session continuity, bootstrap freshness, and tool latency.

It also defines an Eval Ledger boundary under `/v1/evals/*`, keeping telemetry read-only while eval harnesses write append-only eval events.

### 5.9 Spec 114 adds public benchmark/proof planes

Spec 114 defines the public domains:

```text
bench.focusa.dev = public benchmark scoreboard
evals.focusa.dev = technical eval system
proof.focusa.dev = immutable redacted proof receipts
```

It also states that `bench.focusa.dev` must not expose the internal Focusa daemon or `/v1/evals/*` directly to the public internet; public APIs serve generated/redacted artifacts only.

### 5.10 Public proof is deny-by-default

The public stream policy says public cards are deny-by-default and require public-card fields, redaction status, secret scan status, and `publish_allowed=true`.  Spec 114 forbids publishing raw logs, raw prompts, raw diffs unless explicitly public-safe, tokens, secrets, private file contents, sensitive browser diagnostics, unredacted paths, host secrets, and private holdout task bodies.

## 6. Master SaaS architecture

```text
Developer
 ├─ npx focusa
 ├─ ssh cloud.focusa.dev
 └─ cloud.focusa.dev
        │
        ▼
Focusa Cloud Control Plane
 ├─ accounts
 ├─ billing
 ├─ commercial license authority
 ├─ install/update metadata
 ├─ node registry
 ├─ device registry
 ├─ SSH TUI portal
 ├─ MCP client registry
 ├─ tool entitlement registry
 ├─ optional secure relay
 ├─ preload receipt index
 ├─ proof receipt publisher
 ├─ benchmark snapshot index
 ├─ support bundle intake
 ├─ team dashboard
 └─ audit log
        │
        ▼
User-Owned Focusa Node
 ├─ focusa-daemon
 ├─ focusa CLI
 ├─ focusa TUI
 ├─ Pi extension
 ├─ MCP bridge
 ├─ Focusa Tool Gateway
 ├─ Focusa Code Capsule
 ├─ Workpoints
 ├─ Focus State
 ├─ Evidence refs
 ├─ Trajectory
 ├─ Context Authority
 ├─ Eval Ledger
 └─ local project files
```

## 7. Public domains

```text
focusa.dev          Primary marketing site
cloud.focusa.dev    Hosted control plane
install.focusa.dev  Installer, license, update authority
connect.focusa.dev  Pairing and device trust
relay.focusa.dev    Focusa-managed secure relay
mcp.focusa.dev      Cloud-managed MCP policy and routing endpoint
proof.focusa.dev    Redacted proof receipts
bench.focusa.dev    Public benchmark observatory
evals.focusa.dev    Technical eval system
arena.focusa.dev    Public agent-work showcase
forge.focusa.dev    Cohort and training surface
engine.focusa.dev   UIAI Engine companion
```

## 8. First-class entrypoints

### 8.1 `npx focusa`

`npx focusa` is the fastest install and activation path.

It provides:

```text
guided install
license activation
node registration
Pi tool installation
MCP bridge installation
device pairing
doctor checks
proof publishing
benchmark launch
web dashboard opening
SSH portal instructions
```

Required commands:

```bash
npx focusa
npx focusa install
npx focusa cloud connect
npx focusa pi install
npx focusa mcp install
npx focusa mcp start
npx focusa mcp doctor
npx focusa tools doctor
npx focusa proof publish
npx focusa bench smoke
```

`npx focusa` delegates binary installation to `install.focusa.dev` and local daemon execution to the installed Focusa binaries.

### 8.2 `ssh cloud.focusa.dev`

`ssh cloud.focusa.dev` is a terminal-native SaaS cockpit.

It provides:

```text
public demo
install command generator
account login
SSH key enrollment
node list
license state
pairing flows
relay status
proof publishing
benchmark browsing
support workflows
dashboard deep links
```

It is a custom SSH application.

It does not expose:

```text
shell
filesystem
raw daemon
raw project data
raw Workpoints
raw evidence
raw logs
raw Eval Ledger
```

### 8.3 `cloud.focusa.dev`

`cloud.focusa.dev` is the visual dashboard.

It provides:

```text
account
billing
license
nodes
devices
SSH identities
MCP clients
Pi clients
relay status
sync topology
thread ownership
preload receipts
proof receipts
benchmark snapshots
team settings
support bundles
audit logs
```

## 9. Core modules

## 9.1 Install and License Authority

Domain:

```text
install.focusa.dev
```

Responsibilities:

```text
platform-aware install scripts
release metadata
license validation
eval-mode activation
update channels
checksum metadata
signature metadata
commercial entitlement issuance
node registration token issuance
```

Installer requirements:

```text
installs real Rust binaries
downloads correct platform asset
verifies SHA256SUMS
verifies signed artifacts
installs daemon service
performs post-install health check
rolls back failed installs
supports Linux systemd
supports macOS LaunchAgent
supports Windows PowerShell installer
supports stable/preview/nightly channels
writes local license file
validates commercial license with cloud authority
marks evaluation installs as evaluation tier
emits clear recovery hints
```

Cloud stores:

```text
account ID
license hash
license tier
node registration token
release channel
platform target
install outcome summary
```

Cloud does not store:

```text
project files
Workpoints
Focus State
raw logs
private paths
private prompts
```

## 9.2 Node Registry

Domain:

```text
cloud.focusa.dev/nodes
```

Responsibilities:

```text
register user-owned nodes
list nodes
show online/offline state
show installed version
show platform
show license tier
show daemon health
show update availability
show Pi installation status
show MCP installation status
show preload support
show proof support
show relay status
```

Node heartbeat schema:

```json
{
  "schema": "focusa.cloud.node_heartbeat.v1",
  "node_id": "node_123",
  "machine_id": "machine_uuid",
  "version": "0.9.25-dev",
  "platform": "linux-x86_64-glibc",
  "license_tier": "operator",
  "daemon_health": "ok",
  "pi_tools": "installed",
  "mcp_bridge": "not_installed",
  "preload_supported": true,
  "proof_publish_supported": true,
  "evals_supported": false,
  "sync_supported": true,
  "relay_status": "not_connected",
  "last_seen": "2026-06-29T12:00:00Z"
}
```

Heartbeat excludes private cognition.

## 9.3 Pairing and Device Trust

Domain:

```text
connect.focusa.dev
```

Responsibilities:

```text
pair nodes
pair Mac app
pair Pi sessions
pair MCP clients
pair SSH identity
pair browser session
pair support session
revoke devices
rotate tokens
show audit trail
```

Device types:

```text
node
mac_app
pi_session
mcp_client
ssh_identity
browser_session
support_session
```

Token requirements:

```text
scoped
revocable
auditable
device-bound
least-privilege
read-first
mutation-grant explicit
expirable
renewable
```

## 9.4 Secure Relay

Domain:

```text
relay.focusa.dev
```

Focusa Relay exposes Focusa capabilities, not arbitrary ports.

Relay modes:

```text
heartbeat_only
byo_tunnel
focusa_managed_relay
enterprise_self_hosted_relay
```

Launch mode:

```text
heartbeat_only
byo_tunnel
```

Operator Pro beta mode:

```text
focusa_managed_relay
```

Enterprise mode:

```text
enterprise_self_hosted_relay
```

Focusa Managed Relay requirements:

```text
node opens outbound connection
relay authenticates node
browser/session authenticates user
request is checked against route allowlist
node validates token and scope
node validates project authority
node redacts response before cloud display
relay logs request metadata
relay does not expose raw daemon port
relay does not publish raw /v1/*
relay does not bypass local Focusa authority
```

Allowed relay capabilities:

```text
node.health.read
node.version.read
node.update.read
node.license.read
node.devices.read
node.devices.revoke
sync.status.read
sync.topology.read
proof.receipts.list
proof.receipts.preview_redacted
proof.receipts.publish
preload.receipts.list
preload.receipts.read_summary
project.card.read_redacted
workpoint.status.read_redacted
bench.snapshot.publish
support.bundle.request
```

Disallowed relay behavior:

```text
raw_port_forward
raw_daemon_proxy
raw_workpoint_export
raw_focus_state_export
raw_prompt_export
raw_log_export
raw_browser_diagnostics_export
raw_diff_export
raw_env_export
raw_file_content_export
unscoped_mutation
```

## 9.5 Focusa Tool Gateway

Product name:

```text
Focusa Tool Gateway
```

Also exposed as:

```text
Focusa MCP Gateway
```

One-line definition:

```text
Focusa Tool Gateway gives every Focusa tool a cross-platform MCP surface while preserving local node authority.
```

Architecture:

```text
MCP Client
  ↓
focusa-mcp local server
  ↓
Focusa Tool Gateway
  ↓
Local Focusa Daemon
  ↓
Focusa Core / Reducer / Project Files / Evidence
```

Cloud path:

```text
MCP Client
  ↓
focusa-mcp local server
  ↓
Focusa Cloud policy / entitlement / relay
  ↓
User-owned Focusa node
  ↓
Local execution
```

Canonical source of truth:

```text
docs/current/focusa-tool-contracts.json
apps/pi-extension/src/tool-contracts.ts
docs/90-ontology-backed-tool-contracts-parity-spec.md
```

MCP generation rules:

```text
contract.name               → MCP tool name
contract.label              → MCP title
contract.purpose            → MCP description
contract.api_routes         → local daemon route mapping
contract.cli_commands       → CLI fallback mapping
contract.result_envelope    → MCP output schema
contract.side_effect_profile → risk annotation
contract.parity_status      → availability annotation
contract.exemptions         → compatibility note
```

All `focusa_*` Pi tools receive MCP parity.

All `focusa_*` tools become cloud-manageable.

All `focusa_*` tools become npx-installable.

Project-authoritative tools execute on the user-owned Focusa node.

Cloud-native execution is limited to install, license, public proof, public benchmark, billing, account, team, and support workflows.

## 9.6 Tool execution classes

### Class A — Local read tools

Read tools execute locally and may return redacted summaries to Cloud.

Examples:

```text
focusa_project_identity
focusa_project_card
focusa_workpoint_resume
focusa_tool_doctor
focusa_preload_verify
focusa_sync_status
```

### Class B — Local write tools

Write tools execute locally.

They require:

```text
valid device token
project scope
workstream scope
thread authority
side-effect declaration
operator approval when sensitive
audit log
```

Examples:

```text
focusa_workpoint_checkpoint
focusa_project_card_outcome
focusa_evidence_capture
focusa_metacognition_capture
focusa_preload_write
focusa_sync_import
```

### Class C — Code execution tools

Code execution tools execute inside Focusa Code Capsule.

### Class D — Cloud tools

Cloud tools execute in Focusa Cloud.

Allowed cloud tools:

```text
license_validate
install_command_generate
node_register
device_pair
proof_publish_public_safe
benchmark_snapshot_publish
billing_manage
team_manage
support_bundle_request
```

Disallowed cloud tools:

```text
raw_project_read
raw_workpoint_export
raw_focus_state_export
raw_prompt_export
raw_log_export
raw_browser_diagnostics_export
raw_diff_export
arbitrary_shell
```

## 9.7 Focusa Code Capsule

Focusa Code Capsule is the code execution boundary used by MCP tools that need to inspect, test, verify, or preserve project context.

Default execution location:

```text
local user-owned node
```

Cloud execution is allowed only for:

```text
public benchmark fixtures
explicitly uploaded support bundles
public-safe proof rendering
synthetic eval tasks
enterprise-approved sandbox jobs
```

Code Capsule requirements:

```text
project-root scoped
workstream scoped
thread scoped where applicable
network restricted by default
timeout enforced
filesystem allowlist
secret masking
artifact capture
evidence ref creation
operator-visible command preview
tool_result_v1 output
```

Code Capsule output:

```text
stdout summary
stderr summary
exit code
duration
artifact refs
evidence refs
redaction report
tool_result_v1
```

Code Capsule never emits:

```text
unredacted private file dump
secret values
raw environment
unbounded logs
hidden network calls
```

## 9.8 Agent Bootstrap Receipts

Domain:

```text
cloud.focusa.dev/preload
```

Purpose:

```text
prove that an agent session received the correct Focusa startup context
```

Targets:

```text
cursor
claude
codex
pi
opencode
generic
```

Cloud stores receipt summaries, not raw packets by default.

Receipt summary schema:

```json
{
  "schema": "focusa.cloud.preload_receipt_summary.v1",
  "receipt_id": "receipt_123",
  "node_id": "node_123",
  "target": "claude",
  "status": "verified",
  "packet_id": "packet_123",
  "project_scope": "verified",
  "continuity_id_present": true,
  "workpoint_present": true,
  "mission_present": true,
  "exact_next_action_present": true,
  "do_not_drift_present": true,
  "evidence_refs_or_proof_gap_present": true,
  "fail_phrase": "FOCUSA_PRELOAD_FAIL",
  "generated_at": "2026-06-29T12:00:00Z"
}
```

Dashboard card:

```text
Agent Readiness
Cursor: verified
Claude: verified
Codex: missing Workpoint
Pi: active
OpenCode: not configured
```

## 9.9 Multi-Node Registry

A Focusa user may run Focusa daemons on multiple machines.

Supported machines:

```text
MacBook
desktop workstation
VPS
homelab server
CI runner
team shared server
agency client node
```

Each machine runs its own Focusa daemon and local persistence.

Focusa Cloud coordinates those nodes.

Focusa Cloud does not merge canonical cognition in the cloud.

Cloud node schema:

```json
{
  "cloud_node_id": "node_123",
  "machine_id": "local-machine-uuid",
  "account_id": "acct_123",
  "team_id": "team_123",
  "display_name": "Verious MacBook",
  "platform": "macos-aarch64",
  "role": "operator_workstation",
  "status": "online"
}
```

## 9.10 Peer Discovery

Every node-to-node sync relationship is explicit.

Cloud stores peer metadata.

Nodes store local sync state.

Secrets are stored only in approved encrypted secret storage.

Peer auth tokens are never stored as raw Focusa cognitive state.

Peer record schema:

```json
{
  "peer_id": "peer_vps",
  "source_node_id": "node_macbook",
  "target_node_id": "node_vps",
  "project_root_key": "project_hash",
  "workstream_key": "continuity_id",
  "status": "active",
  "last_seen_at": "..."
}
```

## 9.11 Sync Topology

Cloud displays sync topology.

Supported topologies:

```text
single user, two nodes
single user, many nodes
team shared project
agency multi-client workspace
relay daisy chain
```

Topology dashboard shows:

```text
nodes
peers
relay path
last sync
backlog
errors
version mismatch
license mode
platform
project_root_key
workstream_key
thread ownership
```

## 9.12 Scoped CRDT Sync

Sync is scoped by:

```text
project_root_key
workstream_key
machine_id
thread_id
session_id
event_id
```

No cloud or relay operation syncs unscoped cognition.

All multi-node sync operations are project-root and workstream scoped.

CRDT reconciliation reconciles event logs.

CRDT reconciliation does not silently merge canonical cognitive state.

## 9.13 Thread Ownership

A thread is the unit of cognitive ownership.

Cloud dashboard shows:

```text
thread_id
project_root_key
workstream_key
owner_machine_id
owner_node_display_name
attached_sessions
local_role
remote_roles
proposal_count
sync_status
last_canonical_write
last_observation_import
```

Thread roles:

```text
owner
assistant
observer
reviewer
ci_runner
support
```

Only `owner` writes canonical Focus State for that thread.

All other roles generate observations or proposals.

## 9.14 Multiplexed Sessions

A machine may run multiple sessions.

Session types:

```text
Pi session
Claude Code session
Codex session
Cursor session
OpenCode session
MCP client session
manual CLI session
menubar session
CI eval session
```

Each session is attached to:

```text
machine_id
instance_id
session_id
thread_id
project_root_key
workstream_key
role
```

Multiplexing dimensions:

```text
multiple nodes
multiple sessions per node
multiple agents per session
multiple threads per project
multiple workstreams per project
multiple team members per node set
multiple clients through MCP/Pi
```

Cloud does not multiplex by merging cognition.

Cloud multiplexes by routing scoped capability requests to the correct node/thread/session.

Routing key:

```text
team_id
cloud_node_id
machine_id
project_root_key
workstream_key
thread_id
session_id
client_type
capability
```

## 9.15 Team Proposal Inbox

Cloud displays:

```text
non-owner attempted canonical mutation
remote observation suggests state change
ownership transfer request
project scope mismatch
workstream mismatch
stale packet
duplicate event skipped
sync error
tool permission denial
```

Cloud does not approve proposals automatically.

Owner/admin approves or rejects proposals.

## 9.16 Ownership Transfer

Ownership transfer is explicit.

Flow:

```text
1. Non-owner requests ownership transfer.
2. Cloud records request.
3. Current owner receives approval prompt.
4. Owner approves.
5. Node writes explicit ownership-transfer event.
6. Peers sync event.
7. New owner becomes canonical writer.
8. Cloud updates thread ownership display.
```

No automatic ownership changes.

## 9.17 Proof Receipts

Domain:

```text
proof.focusa.dev
```

Purpose:

```text
host immutable, redacted proof receipts for agent work
```

Receipt types:

```text
workpoint_proof
release_proof
benchmark_proof
browser_diagnostics_proof
agent_preload_proof
client_delivery_proof
```

Publication states:

```text
draft_private
redaction_pending
publish_blocked
private_link
public_snapshot
revoked
```

Proof page fields:

```text
schema
receipt_id
receipt_type
redacted_project_label
redacted_scope_id
canonical_status
tool_family
evidence_refs_public_safe
redaction_status
secret_scan_status
publish_allowed
source_version
generated_at
limitations
```

## 9.18 Benchmark Observatory

Domains:

```text
bench.focusa.dev
evals.focusa.dev
```

Separation:

```text
bench.focusa.dev = public scoreboard
evals.focusa.dev = technical eval system
proof.focusa.dev = receipt viewer
```

Public story:

```text
Same agent.
Same task.
Focusa ON vs Focusa OFF.
Measured results only.
```

Benchmark requirements:

```text
full_focusa vs no_focusa is headline comparison
passive_focusa and tool_only_focusa are diagnostic ablations
public claims require completed Eval Ledger run
public claims require public-safe proof bundle
private holdout task bodies are never public
raw metrics display beside composite scores
failures become improvement candidates
improvements require reruns
release claims require measured evidence
```

## 9.19 Commercial Licensing and Support

Focusa source-available licensing already requires a paid commercial license for company/team use, client delivery, production use, hosted service use, embedding, redistribution, resale, and internal commercial operations.

Focusa Cloud operationalizes:

```text
commercial entitlement
license validation
support tier
team rights
custom adapter rights
client delivery rights
managed relay rights
proof hosting rights
benchmark hosting rights
```

## 10. Data policy

### 10.1 Cloud stores by default

```text
account
billing profile
license tier
node ID
device ID
SSH identity fingerprint
version
platform
heartbeat
feature flags
install status
update status
pairing status
relay status
sync topology metadata
redacted receipt metadata
public-safe proof snapshots
benchmark public-safe snapshots
support ticket metadata
```

### 10.2 Cloud stores only after explicit user action

```text
private proof receipt
public proof receipt
redacted preload receipt summary
redacted project display label
benchmark public-safe snapshot
support bundle
support session grant
```

### 10.3 Cloud never stores by default

```text
code
raw prompts
raw Workpoints
raw Focus State
raw Evidence payloads
private file paths
private browser diagnostics
raw logs
secrets
tokens
environment variables
raw diffs
private Eval Ledger rows
holdout benchmark task bodies
```

## 11. Privacy modes

### 11.1 Local-only

```text
Cloud account exists.
No node data is sent.
Install and license only.
```

### 11.2 Heartbeat

```text
Node sends safe health metadata.
Cloud dashboard shows status.
No private cognition is sent.
```

### 11.3 Relay

```text
Node opens outbound connection.
Cloud routes allowlisted capability requests.
Node validates authority.
Node redacts response.
Cloud displays scoped result.
```

Relay never exposes arbitrary local ports.

Relay never exposes raw daemon API.

## 12. Security requirements

### 12.1 SSH portal

```text
custom SSH app
no shell access
rate limiting
SSH key enrollment
account linking
device revocation
session audit
install command generator
dashboard links
```

### 12.2 MCP Gateway

```text
tool contract generated
tool input schema validated
tool output wrapped in tool_result_v1
side-effect profile enforced
dangerous tools approval-gated
device token required
node scope required
thread authority required for writes
audit log required
```

### 12.3 Relay

```text
route allowlist
capability allowlist
no raw port forwarding
no raw daemon proxy
node-side authority check
node-side redaction
cloud-side audit
support session expiry
```

### 12.4 Support bundles

```text
opt-in only
node-generated
redacted before upload
manifest included
automatic expiry
revocable by user
```

## 13. Team model

Team roles:

```text
owner
admin
operator
developer
observer
client_viewer
support
```

Capabilities:

```text
node.read
node.register
node.revoke
peer.read
peer.manage
sync.read
sync.trigger
thread.read
thread.own
thread.transfer
proposal.read
proposal.approve
proof.preview
proof.publish
relay.use
support.grant
mcp.client.manage
mcp.tool.enable
mcp.tool.disable
```

## 14. Pricing

### 14.1 Evaluation

```text
Free
```

Includes:

```text
local self-hosted Focusa evaluation
eval license
local daemon
local CLI/TUI
local proof preview
npx focusa
public ssh cloud.focusa.dev demo
```

Limits:

```text
no commercial production use
no managed relay
no hosted private proof receipts
no team dashboard
```

### 14.2 Focusa Operator

```text
$49/month
$499/year
```

For:

```text
serious solo builders
indie hackers
agentic coding operators
technical founders
```

Includes:

```text
commercial solo license
1 self-hosted node
cloud heartbeat dashboard
license activation
node registry
npx focusa onboarding
ssh cloud.focusa.dev authenticated cockpit
BYO tunnel support
Pi install helper
MCP install helper
agent preload receipt dashboard
single-node MCP gateway
25 private proof receipts per month
5 public proof receipts per month
basic support bundle upload
```

### 14.3 Focusa Operator Pro

```text
$149/month
$1,499/year
```

For:

```text
power users
solo consultants
high-output builders
AI-assisted product operators
```

Includes:

```text
3 self-hosted nodes
managed relay beta
multi-node sync dashboard
thread ownership view
Pi client management
MCP client management
device registry
scoped token management
tool-family controls
100 private proof receipts per month
25 public proof receipts per month
private proof links
benchmark smoke runs
support bundle upload
priority issue triage
```

### 14.4 Focusa Agency

```text
$299/month base
$29/month per additional seat
$2,999/year base
```

For:

```text
agencies
AI coding consultants
client delivery teams
```

Includes:

```text
commercial client-delivery rights
10 self-hosted nodes
team dashboard
device registry
sync topology
proposal inbox
proof approval workflow
client proof portals
500 private proof receipts per month
100 public proof receipts per month
managed relay
Pi/MCP client scope controls
agency support bundle workflow
benchmark smoke runs
priority onboarding
```

### 14.5 Focusa Growth Team

```text
$799/month
$7,999/year
```

For:

```text
small engineering teams using agents seriously
```

Includes:

```text
25 nodes
15 seats
managed relay
private eval summaries
team benchmark history
audit logs
role-based access
thread ownership transfer
team proof approval
2,000 private proof receipts per month
250 public proof receipts per month
priority support
```

### 14.6 Focusa Enterprise

```text
Starts at $25,000/year
```

Includes:

```text
private control plane option
enterprise self-hosted relay
SSO/SAML/OIDC
custom retention
custom redaction rules
private benchmark suite
private proof observatory
custom Pi/MCP adapters
private MCP gateway
legal/security review support
SLA
onboarding
procurement-ready license terms
```

## 15. Add-ons

```text
Extra node: $15/month
Extra seat: $29/month
Extra private proof receipts: $10 per 100
Extra public proof receipts: $25 per 100
Managed relay overage: $0.15/GB
Benchmark smoke run: $49/run
Full benchmark suite run: $199/run plus model/provider cost
Private onboarding session: $750
Agency client portal: $99/month
Enterprise support block: custom
```

## 16. Launch offers

### 16.1 Founding Operator

```text
$499/year
```

Includes:

```text
Operator plan
founder pricing lock for 12 months
1 onboarding session
early managed relay beta access
MCP bridge access
```

### 16.2 Founding Agency

```text
$1,999/year
```

Includes:

```text
Agency plan
5 nodes during founder term
client proof portals
priority onboarding
managed relay beta access
250 private proof receipts per month
founder pricing lock for 12 months
```

## 17. Implementation plan

This spec does not replace the current Specs 109–114 implementation order. The current order already places Spec 112 installer/platform blockers first, Spec 109 API authority and Spec 114 Eval Ledger next, Spec 111 bootstrap and Spec 110 reminder behavior after that, then Spec 113 benchmark runner, Spec 114 public snapshots, and finally the public observatory UI.

Spec 115 is implemented as the SaaS umbrella around those waves.

### 17.1 Phase A — Installer and license foundation

Required deliverables:

```text
real binary installer
Linux systemd install
macOS LaunchAgent install
Windows PowerShell installer
SHA256SUMS
signed artifacts
license activation
eval mode
post-install health check
rollback
focusa update
stable/preview/nightly channels
```

### 17.2 Phase B — Cloud foundation

Required deliverables:

```text
account system
billing
license authority
node registration
node heartbeat
basic dashboard
public SSH TUI
npx focusa launcher
data boundary page
privacy policy
commercial terms
```

### 17.3 Phase C — Pairing and device trust

Required deliverables:

```text
node pairing
device pairing
SSH identity pairing
token revocation
scoped tokens
audit log
pairing recovery flow
```

### 17.4 Phase D — Relay foundation

Required deliverables:

```text
outbound node connector
route allowlist
scoped capability requests
no raw port forwarding
no raw daemon proxy
relay audit logs
BYO tunnel docs
managed relay status dashboard
```

### 17.5 Phase E — Tool Gateway and MCP parity

Required deliverables:

```text
focusa-mcp local server
tool contract to MCP generator
tool_result_v1 MCP output schema
tool side-effect annotations
local daemon route mapping
CLI fallback mapping
MCP doctor
client config generator
Cloud MCP client registry
tool-family entitlements
MCP client revocation
```

### 17.6 Phase F — Code Capsule

Required deliverables:

```text
local execution capsule
project-root allowlist
workstream scope
thread scope
network restriction
timeout enforcement
secret masking
artifact capture
evidence ref generation
operator-visible command preview
redaction report
```

### 17.7 Phase G — Preload receipts

Required deliverables:

```text
preload build
preload verify
receipt summary
cloud receipt upload
target matrix
failure reason display
agent readiness dashboard
```

### 17.8 Phase H — Multi-node and team sync

Required deliverables:

```text
node topology dashboard
peer discovery
sync status dashboard
CRDT scope display
thread ownership dashboard
proposal inbox
ownership transfer workflow
multiplexed session view
team permissions
agency client isolation
```

### 17.9 Phase I — Proof receipts

Required deliverables:

```text
local proof preview
redaction scanner
secret scanner
receipt schema
private proof links
public proof snapshots
revoke flow
immutable snapshot hash
proof.focusa.dev
```

### 17.10 Phase J — Benchmark observatory

Required deliverables:

```text
Eval Ledger API
benchmark runner
smoke suite
public-safe snapshot generator
bench.focusa.dev
evals.focusa.dev
proof.focusa.dev integration
measured-claim policy enforcement
```

### 17.11 Phase K — SSH TUI

Required deliverables:

```text
custom SSH app
no shell access
rate limiting
public demo mode
account login
SSH key enrollment
install command generator
node cockpit
pairing flows
dashboard deep links
proof publishing
benchmark browsing
support flows
```

## 18. Acceptance criteria

### 18.1 Cloud control plane

```text
User can create account
User can activate license
User can register node
Node sends heartbeat
Cloud shows node online
Cloud shows version/platform/license
Cloud does not receive raw project cognition
```

### 18.2 npx

```text
npx focusa launches
npx focusa install starts installer
npx focusa cloud connect registers node
npx focusa mcp install configures local MCP
npx focusa tools doctor validates contracts
```

### 18.3 SSH

```text
ssh cloud.focusa.dev launches custom TUI
SSH portal exposes no shell
SSH portal supports public demo
SSH portal supports account login
SSH portal generates install commands
SSH portal shows node cockpit
```

### 18.4 Relay

```text
Node connects outbound
Relay allows only capability routes
Relay blocks raw daemon proxy
Node validates scope and token
Node redacts response
Cloud displays scoped result
```

### 18.5 MCP parity

```text
All focusa_* tools have generated MCP definitions
All MCP definitions come from canonical tool contracts
All MCP outputs map to tool_result_v1
Dangerous tools require approval
Local-authority tools execute locally
Cloud tools are limited to cloud-safe workflows
```

### 18.6 Multi-node

```text
Two nodes register under one account
Peer relationship is visible
Sync status shows cursor/backlog
CRDT export/import is scoped
Remote events import as observations
Thread owner remains canonical writer
Proposal inbox displays conflicts
```

### 18.7 Teams

```text
Team has multiple users
Team has multiple nodes
Team permissions govern node/tool/proof access
Thread ownership is visible
Ownership transfer is explicit
Client workspaces are isolated
```

### 18.8 Proof

```text
Proof receipt can be generated locally
Redaction scan runs
Secret scan runs
Private proof link can be created
Public proof requires publish_allowed=true
Proof can be revoked
```

### 18.9 Bench

```text
Smoke benchmark runs
Eval Ledger receives append-only events
Public-safe snapshot is generated
bench.focusa.dev displays measured results
Public claims link to proof receipt
```

## 19. Public messaging

### 19.1 Primary headline

```text
Cloud command center for self-hosted AI coding infrastructure.
```

### 19.2 Secondary headline

```text
Install with npx. Operate over SSH. Prove agent work without giving up local control.
```

### 19.3 Trust headline

```text
Your Focusa node owns the work. Focusa Cloud coordinates access, proof, licensing, and visibility.
```

### 19.4 Developer CTA

```bash
npx focusa
```

### 19.5 Terminal CTA

```bash
ssh cloud.focusa.dev
```

### 19.6 Proof CTA

```text
Publish redacted proof receipts at proof.focusa.dev.
```

### 19.7 Benchmark CTA

```text
See Focusa ON vs Focusa OFF at bench.focusa.dev.
```

## 20. Terms to use

Use:

```text
self-hosted node
hosted control plane
tool gateway
MCP parity
code capsule
proof receipt
agent preload receipt
secure relay
node registry
device trust
thread ownership
sync topology
local authority
redacted snapshot
measured benchmark
cloud coordination
local-first
```

Avoid:

```text
hosted memory
cloud brain
sync all state
upload project
generic tunnel
raw daemon proxy
cloud agent runtime
unlimited observability
automatic productivity claims
```

## 21. Product moat

Focusa Cloud combines:

```text
self-hosted local authority
cloud coordination
npx onboarding
SSH terminal dashboard
Focusa-aware secure relay
Pi/MCP client management
MCP parity for focusa_* tools
code execution capsule
agent bootstrap receipts
proof receipts
benchmark observatory
multi-node CRDT sync visibility
thread ownership
team proposal workflows
measured claim discipline
commercial licensing
```

The moat is not tunneling.

The moat is agentic coding continuity, proof, authority, and local-first trust.

## 22. Final product shape

```text
Focusa Cloud
= hosted command center for self-hosted Focusa nodes

npx focusa
= fastest install and activation path

ssh cloud.focusa.dev
= terminal-native SaaS cockpit

cloud.focusa.dev
= visual dashboard

connect.focusa.dev
= pairing and device trust

relay.focusa.dev
= Focusa-aware secure relay

mcp.focusa.dev
= cloud-managed MCP policy and routing endpoint

proof.focusa.dev
= redacted proof receipts

bench.focusa.dev
= public measured evidence

evals.focusa.dev
= technical eval system
```

## 23. Final rule

Every cloud feature must strengthen the local-first promise.

Cloud convenience is the product.

Local authority is the trust.

Focusa tools become universal.

Focusa authority stays local.

MCP is the compatibility layer.

Cloud is the control plane.

`npx` is the installer.

SSH is the terminal cockpit.

Code Capsule is the execution boundary.

Proof receipts are the public trust layer.

Benchmarks are the measured claim layer.
