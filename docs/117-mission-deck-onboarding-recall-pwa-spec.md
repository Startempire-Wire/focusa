# Spec 117 — Focusa Mission Deck, Guided Onboarding, Recall, and PWA Surface

## 0. Status

**Status:** proposed / review draft
**Spec number:** 117
**Owner:** Verious Smith
**Scope:** Focusa TUI evolution, first-run onboarding, repeatable product education, guided walkthroughs, visual Mission Deck language, Focusa Recall integration, PWA/browser Mission Deck, optional terminal-in-browser bridge, launch/release proof gates.

**Product surface name:** Focusa Mission Deck
**Short form:** Mission Deck / Deck
**Existing command:** `focusa-tui` remains supported
**Future friendly alias:** `focusa deck`

**One-line promise:**

> Focusa Mission Deck shows what the agent is doing, what is proven, what is stale, and what the next safe move is.

**Launch promise:**

> In five minutes, a new operator can bind a project, create a Workpoint, attach proof, search prior agent work, and understand how Focusa keeps the mission alive.

---

## 1. Normative Basis

This spec extends existing Focusa architecture. It does **not** redefine Workpoints, Evidence, ProjectIdentity, Trajectory, Context Authority, Context Cognition, install authority, or public proof rules.

It is grounded against:

| Existing surface                        | This spec uses it for                                                                     |
| --------------------------------------- | ----------------------------------------------------------------------------------------- |
| Existing Focusa README/runtime          | Local-first daemon, API, CLI, TUI, Pi extension, menubar product shape                    |
| Existing TUI spec                       | Ratatui/TUI principles: calm, introspective, real-time, non-invasive                      |
| Context Cognition spec                  | Advisory context, contradiction flags, selected context, route guidance                   |
| Spec-first lifecycle / claim discipline | No implementation or closure claim without spec, task decomposition, and proof            |
| Agent bootstrap/delivery specs          | Mission/Workpoint/Trajectory/Evidence packets for cold agent sessions                     |
| Install architecture specs              | Install/update path for `focusa`, `focusa-daemon`, and `focusa-tui`                       |
| Public benchmark/proof specs            | Redacted public proof snapshots and measured-claim discipline                             |
| Cloud/control-plane specs               | Local-first SaaS boundary, device trust, pairing, PWA access, proof receipts              |
| Authority model                         | Operator steering, ProjectIdentity, Continuity ID, Workpoint, Evidence, Context Authority |
| Workpoint/session scope guard           | `project_root + continuity_id` authority boundary                                         |

**Hard rule:** any mismatch between this spec and current code is an implementation gap, not permission to weaken the spec.

---

## 2. Problem Statement

Focusa already has deep runtime substance: daemon, API, CLI, TUI, Workpoints, Evidence Refs, Context Authority, Pi integration, and release/install paths.

The launch risk is not lack of power.

The launch risk is this:

```text
A buyer installs Focusa.
They open the TUI.
They see many powerful tabs.
They do not know what to do first.
They do not feel the mission-continuity value within five minutes.
```

The current TUI is useful for operators who already understand Focusa. It is not yet optimized as the first product encounter for a buyer, evaluator, or new operator.

This spec turns the existing TUI into **Focusa Mission Deck**: a terminal-native and eventually browser/PWA mission surface that teaches and governs the core product loop:

```text
Bind project
→ create Workpoint
→ attach evidence
→ search prior agent work
→ evaluate scope/proof/freshness
→ resume safely
```

---

## 3. Core Thesis

The first Focusa experience must not be documentation-first.

It must be process-first:

```text
Show the mission.
Show the boundary.
Show the Workpoint.
Show the proof gap.
Show the next safe action.
Let the user perform it.
```

The operator should not need to understand every Focusa term before seeing value.

Mission Deck should make the product explain itself through live state, visual walkthroughs, and safe next actions.

---

## 4. Product Positioning

Do **not** position Mission Deck as a generic dashboard.

Do **not** position it as a terminal emulator clone.

Position it as:

```text
Mission control for AI-agent continuity.
```

More specifically:

```text
A mission-aware terminal/PWA surface for Workpoints, Recall, Evidence, drift, recovery, and next safe action.
```

Scritty validates that developers want terminal-native, browser-accessible, cross-agent memory surfaces.

Focusa’s differentiation is stronger:

```text
Memory is not enough.

Focusa shows:
- what still matters,
- what is scoped,
- what is proven,
- what is stale,
- what is contradicted,
- and what the agent is allowed to do next.
```

---

## 5. Non-Goals

This spec is not:

* a replacement for the existing `focusa-tui` binary;
* a rename that breaks existing release assets;
* a raw shell/PTY bridge as the first browser surface;
* a generic cloud memory service;
* a Scritty clone;
* a replacement for Workpoint, Evidence, Trajectory, ProjectIdentity, Context Cognition, or Context Authority;
* a prompt-stuffing mechanism;
* a new source of canonical truth;
* a bypass around spec-first lifecycle gates;
* a bypass around install/checksum/license/update rules;
* a public proof surface that exposes raw logs, raw prompts, raw diffs, private paths, secrets, or unredacted diagnostics.

---

## 6. Product Vocabulary

### 6.1 Mission Deck

The operator-facing TUI/PWA surface for understanding and steering Focusa.

Mission Deck is the product name. The binary remains `focusa-tui` for compatibility.

### 6.2 First Encounter

The first guided screen shown when a user opens Mission Deck with no verified project-bound Workpoint.

### 6.3 Walkthrough

A repeatable, state-aware product education flow.

Walkthroughs are not one-time tutorials. They can be repeated for onboarding, recovery, release readiness, agent handoff, and proof discipline.

### 6.4 Mission Ladder

Visual representation of:

```text
HLT
 └─ MLG
     └─ STG
         └─ Workpoint
             └─ Evidence
```

### 6.5 Next Safe Action

A single recommended action generated from current Focusa state, authority posture, and walkthrough context.

### 6.6 Recall Card

A search result from Focusa Recall that includes source, scope, freshness, memory status, proof status, and allowed use.

### 6.7 Proof Meter

A compact visual proof status:

```text
none      [-----]
linked    [##---]
verified  [#####]
```

### 6.8 Beginner Mode

A simplified, plain-language mode for new users and evaluators.

### 6.9 Operator Mode

A dense, lazygit-style mode for experienced Focusa users.

---

## 7. Current TUI Inventory

The existing TUI already has valuable foundations:

```text
crates/focusa-tui
focusa-tui binary
ratatui/crossterm terminal UI
Focusa API polling
multiple runtime tabs
read-only inspection posture
```

Current gaps:

```text
no beginner-first default home
no guided first-run process
no walkthrough engine
no repeatable product education layer
no plain-language concept overlays
no command palette for next safe action
no Recall search surface
no Mission Ladder visual
no proof meter visual
no browser/PWA Mission Deck
no launch-demo acceptance path proving a buyer can understand Focusa in five minutes
```

---

## 8. Experience Principles

### 8.1 One Next Step

Beginner Mode must show one primary next safe action, not a list of ten options.

### 8.2 Explain in Place

Every unfamiliar term must be explainable with `h` or a visible help affordance.

### 8.3 Authority Visible Everywhere

Every screen that can influence action must show one authority badge:

```text
[ok]
[advisory]
[blocked]
[stale]
[proof missing]
[global advisory]
```

### 8.4 Proof Before Done

If work is claimed complete without evidence, Mission Deck must show a proof gap.

### 8.5 Recall Is Not Authority

Recall results can inform the operator or agent. They cannot directly become canonical continuation truth.

### 8.6 Recovery Beats Mystery

Disconnected, blocked, stale, unbound, no-Workpoint, no-evidence, and scope-mismatch states must always show a recovery hint.

### 8.7 Visual First, Docs Second

Use cards, ladders, meters, badges, and guided actions before sending the user to docs.

---

## 9. First Encounter Flow

When a user runs:

```bash
focusa-tui
```

or future alias:

```bash
focusa deck
```

Mission Deck chooses a starting state.

### 9.1 State Decision Tree

```text
Daemon unreachable
  → show Start/Recover card

Daemon reachable, version mismatch
  → show Upgrade/Recover card

Daemon reachable, license/eval unknown
  → show License/Eval card

Daemon healthy, project unbound
  → show Bind Project card

Project bound, no Workpoint
  → show First Workpoint card

Workpoint exists, no Evidence
  → show Attach Proof card

Workpoint exists, evidence linked
  → show Mission Resumable card

Scope mismatch or stale context
  → show Drift Recovery card
```

### 9.2 First Encounter Card

```text
┌ Focusa Mission Deck ──────────────────────────────────────────────┐
│ AI agents lose the mission across long sessions and handoffs.      │
│ Focusa keeps the project, goal, Workpoint, evidence, and next step. │
├────────────────────────────────────────────────────────────────────┤
│ Start here                                                         │
│ 1. Bind this project                                                │
│ 2. Create your first Workpoint                                      │
│ 3. Attach one proof item                                            │
│ 4. Resume the mission like a new agent would                        │
├────────────────────────────────────────────────────────────────────┤
│ n next safe step   / search recall   h explain   d doctor   q quit │
└────────────────────────────────────────────────────────────────────┘
```

### 9.3 First-Run Success Condition

The first encounter is complete when the user has:

```text
verified project root
continuity id
Workpoint or Workpoint candidate
at least one Evidence Ref or explicit proof gap
rendered next safe action
visible explanation of how an agent can resume
```

---

## 10. Beginner Mode

Beginner Mode is the default when any of these are true:

```text
no project identity
no Workpoint
first install marker missing
first-run walkthrough incomplete
user explicitly toggles beginner mode
```

### 10.1 Beginner Copy Rules

Replace internal error phrasing with plain language.

```text
Internal:
no canonical Workpoint for verified project_root + continuity_id

Beginner:
Focusa does not yet know what project this mission belongs to. Bind this folder first.
```

```text
Internal:
evidence refs missing

Beginner:
The agent says work is done, but Focusa has no proof yet. Attach a test, file, screenshot, or command output.
```

```text
Internal:
action_authority_for_current_ask=false

Beginner:
This saved mission belongs to a different scope than your current request. Review or rebind before changing files.
```

```text
Internal:
daemon unreachable

Beginner:
The Focusa background service is not running. Start it, then reopen Mission Deck.
```

### 10.2 Beginner Commands Shown Inline

Beginner Mode may show commands, but only after explaining why.

```bash
focusa start
focusa doctor --scope host
focusa onboard --scope project
focusa status --operator
focusa recover --dry-run
```

### 10.3 Beginner Exit Criteria

Mission Deck can suggest Operator Mode after:

```text
first mission walkthrough complete
user has resumed a Workpoint at least once
user has attached evidence or intentionally acknowledged a proof gap
user has seen the authority badge legend
```

---

## 11. Operator Mode

Operator Mode is dense and keyboard-first.

Default advanced layout:

```text
┌ Projects / Continuities ───┬ Mission Ladder ─────────────┬ Recall ────────────────┐
│ focusa                     │ HLT: Ship Operator Preview  │ / installer signing    │
│ uiai-engine                │ MLG: Installer reliability  │ Claude decision        │
│ forge.focusa.dev           │ STG: close recover/upgrade  │ Codex test failure     │
│ arena.focusa.dev           │ Workpoint: active           │ Pi resume packet       │
├ Audit Timeline ────────────┼ Evidence ───────────────────┼ Next Safe Action ──────┤
│ 05:31 upgrade added        │ test: static PASS           │ n resume Workpoint     │
│ 05:23 recover added        │ commit: 3894e884            │ e attach evidence      │
│ 19:37 symlink fix          │ release failure linked      │ d run doctor           │
└ / search  n next  h explain  w workpoint  e evidence  d doctor  q quit ──────────┘
```

Operator hotkeys:

```text
/ = Mission Recall
n = next safe action
h = explain current card
w = Workpoint
r = resume
p = proof/evidence
m = mark memory status
d = doctor/recover
o = operator/beginner toggle
g = go to tab/menu
q = quit
```

---

## 12. Walkthrough Engine

Walkthroughs are state-aware process definitions.

### 12.1 Schema

```yaml
Walkthrough:
  schema_version: focusa.walkthrough.v1
  id:
  title:
  audience: beginner | operator | agent | evaluator
  trigger:
    first_run: true | false
    missing_project: true | false
    missing_workpoint: true | false
    missing_evidence: true | false
    scope_mismatch: true | false
    release_mode: true | false
  goal:
  why_it_matters:
  required_state:
    daemon: reachable | optional
    project_identity: required | optional
    workpoint: required | optional
    evidence: required | optional
  steps:
    - id:
      title:
      explanation:
      visual:
      action_kind: read | propose | write | external
      command:
      api_route:
      authority_required:
      success_condition:
      recovery_hint:
  completion:
    success_message:
    proof_required:
    evidence_class: actual | partial | surrogate | blocked | missing
  resettable: true
  side_effects: []
```

### 12.2 Walkthrough Storage

Walkthrough progress is local-first and project-aware.

Suggested storage:

```text
~/.focusa/deck/walkthroughs/{project_hash}.jsonl
```

Each event includes:

```yaml
WalkthroughEvent:
  walkthrough_id:
  step_id:
  project_root:
  continuity_id:
  event_type: started | advanced | completed | reset | blocked
  timestamp:
  evidence_refs: []
  authority_posture:
```

### 12.3 Walkthrough Surfaces

```text
TUI Mission Deck
CLI: focusa deck walkthrough ...
API: /v1/deck/walkthroughs/*
PWA: /deck/walkthroughs
Pi tool later: focusa_deck_walkthrough
```

---

## 13. Required Walkthroughs

### 13.1 First Mission

Purpose: teach the core Focusa loop.

Steps:

```text
1. Start daemon
2. Bind project
3. Create Workpoint
4. Attach evidence or acknowledge proof gap
5. Resume Workpoint
6. Show “mission is resumable”
```

Success message:

```text
This mission can now survive handoff, compaction, and agent restart.
```

### 13.2 Agent Handoff

Purpose: show why Focusa exists.

Steps:

```text
1. Show current mission
2. Show current Workpoint
3. Render agent handoff / bootstrap packet
4. Show what a new agent receives
5. Show drift boundaries
6. Show evidence/proof expectations
```

### 13.3 No Proof, No Done

Purpose: teach evidence discipline.

Steps:

```text
1. Display an agent completion claim
2. Check evidence refs
3. Show proof gap if missing
4. Attach proof or mark proof intentionally missing
5. Re-render proof meter
```

### 13.4 Mission Recall

Purpose: integrate Scritty-style cross-agent memory without weakening Focusa authority.

Steps:

```text
1. Search prior agent sessions
2. Show source/provider/session/scope
3. Show active/stale/superseded/contradicted status
4. Show proof status
5. Promote to Workpoint candidate only after verification
```

### 13.5 Recover After Drift

Purpose: show Focusa safety value.

Steps:

```text
1. Detect mismatched project or continuity
2. Explain the mismatch
3. Show blocked/advisory badge
4. Offer safe rebind or resume path
5. Require operator confirmation before mutation
```

### 13.6 Ship Readiness

Purpose: support first launch/release/GTM.

Steps:

```text
1. Show active Workpoint
2. Show open proof gaps
3. Run preflight/doctor
4. Show blocked actions in plain language
5. Show release-safe path
6. Produce final proof card
```

### 13.7 Browser/PWA Pairing

Purpose: teach local-first remote visibility.

Steps:

```text
1. Explain local daemon ownership
2. Start pairing room
3. Show QR/code
4. Pair browser/PWA device
5. Show read-only Mission Deck Web
6. Explain mutation gates and revocation
```

---

## 14. Visual Grammar

### 14.1 Mission Ladder

```text
HLT: Ship Focusa Operator Preview
 └─ MLG: Make install + first run reliable
     └─ STG: Create first mission walkthrough
         └─ Workpoint: active / resumable
             └─ Evidence: linked / missing / verified
```

### 14.2 Scope Badge

```text
[ok]              exact project_root + continuity_id match
[advisory]        useful context, not action authority
[blocked]         unsafe, mismatched, or requires verification
[stale]           older than current state or superseded
[proof missing]   claim lacks evidence refs
[global advisory] result came from widened search
```

### 14.3 Recall Card

```text
Claude Code · current project · 2 days ago
Decision: installer should delegate service rendering to Rust
Status: active
Proof: linked
Use: include after Workpoint verification
```

### 14.4 Drift Warning

```text
This memory came from another project or continuity.
It may help explain history, but it cannot drive the next action.
```

### 14.5 Proof Meter

```text
Proof: none      [-----]
Proof: linked    [##---]
Proof: verified  [#####]
```

### 14.6 Release Readiness Card

```text
Release readiness
- Install path: verified / blocked / missing
- TUI first-run: verified / blocked / missing
- License/eval: verified / blocked / missing
- Recovery: verified / blocked / missing
- Public claims: measured / predicted / blocked
```

---

## 15. Mission Recall Integration

Mission Deck must include Focusa Recall, but Recall remains advisory.

### 15.1 Search Behavior

Mission Recall searches:

```text
Focusa events
Workpoints
Evidence refs
Audit timeline
Agent bootstrap packets
Pi/Codex/Claude/Cursor/OpenCode imports
UIAI diagnostics packets
manual session notes
```

### 15.2 Result Labels

Each result includes:

```yaml
RecallDeckCard:
  result_id:
  provider:
  source_session_id:
  project_root:
  continuity_id:
  timestamp:
  span_type:
  memory_status: active | stale | superseded | contradicted | noise | quarantined
  scope_status: current | same_project_other_continuity | other_project | global_advisory
  proof_status: none | linked | verified
  allowed_use: include | inspect_only | verify_first | exclude
  safe_excerpt:
  evidence_refs: []
  next_action:
```

### 15.3 Promotion Rule

Recall may propose a Workpoint candidate.

Recall must not directly create canonical Workpoint authority.

Promotion flow:

```text
Recall search
→ RecallDeckCard
→ operator selects candidate
→ verify project_root + continuity_id
→ Context Authority preflight
→ evidence/proof check
→ Workpoint candidate render
→ operator approval
→ canonical Workpoint checkpoint
```

---

## 16. Mission Contract Education

Mission Deck must teach the Mission Contract concept as the Focusa-native answer to generic prompt rules.

Mission Contract is generated from:

```text
current operator ask
ProjectIdentity
Continuity ID
HLT / MLG / STG
active Workpoint
Evidence Refs
Context Authority verdict
safe Recall context
blocked actions
drift boundaries
next tools
```

It is displayed as:

```yaml
MissionContractCard:
  mission:
  project:
  continuity:
  next_action:
  evidence_required:
  do_not_drift:
  allowed_tools: []
  blocked_actions: []
  recall_context: []
  authority_badge:
```

Mission Contract output is advisory until bound to current Workpoint authority.

---

## 17. Browser/PWA Mission Deck

### 17.1 Product Decision

Focusa should support a browser/PWA Mission Deck.

Focusa should **not** initially expose a raw shell or raw PTY through the browser.

The first browser/PWA surface is:

```text
safe mission viewer + guided action surface
```

not:

```text
browser terminal with unrestricted shell
```

### 17.2 Local Daemon Routes

Add daemon-served PWA routes:

```text
GET /deck
GET /deck/manifest.json
GET /deck/sw.js
GET /deck/assets/*
GET /deck/pair
```

### 17.3 API Routes

Add read-first Deck API routes:

```text
GET  /v1/deck/home
GET  /v1/deck/mission
GET  /v1/deck/walkthroughs
GET  /v1/deck/walkthroughs/{id}
POST /v1/deck/walkthroughs/{id}/advance
POST /v1/deck/walkthroughs/{id}/reset
GET  /v1/deck/recall/search
GET  /v1/deck/release-readiness
GET  /v1/deck/first-run
```

Mutation routes require Context Authority preflight and explicit operator action.

### 17.4 PWA Requirements

Mission Deck Web must be:

```text
installable
phone-friendly
touch-friendly
offline-aware for docs/walkthrough explainers
live when daemon is reachable
paired by QR/code
token-scoped
read-only by default
action-gated for mutations
revocable
```

### 17.5 Pairing / Security

Use existing Focusa device pairing concepts:

```text
daemon starts pairing
browser/phone opens pair URL or scans QR
operator confirms pairing
daemon mints token
device stores token
Mission Deck Web uses scoped token
operator may revoke
```

Remote browser access must prefer:

```text
Tailscale / private network
or Focusa Cloud relay when implemented
```

Public internet exposure is not the default.

---

## 18. Optional Terminal-in-Browser Bridge

### 18.1 Future Mode

A later phase may add:

```text
GET /terminal
```

using a browser terminal library and a daemon-owned PTY bridge.

### 18.2 Default Policy

Terminal-in-browser starts as:

```text
read-only terminal mirror
```

Write access requires:

```text
paired device
short-lived token
explicit operator approval
scope boundary
Context Authority gate
visible audit event
revocation path
```

### 18.3 Safer First Terminal Commands

Prefer Focusa command palette actions before arbitrary shell:

```text
focusa status --operator
focusa doctor --scope host
focusa recover --dry-run
focusa audit --limit 100
focusa workpoint resume
```

Raw shell write mode is post-GTM unless explicitly approved by operator.

---

## 19. CLI Surface

### 19.1 Existing Command

```bash
focusa-tui
```

must remain supported.

### 19.2 Future Aliases

```bash
focusa deck
focusa deck --mode beginner
focusa deck --mode operator
focusa deck web
focusa deck walkthrough list
focusa deck walkthrough start first-mission
focusa deck walkthrough reset first-mission
focusa deck open
focusa deck terminal --read-only
```

### 19.3 CLI Behavior

`focusa deck` launches the native TUI if available.

If TUI binary is missing:

```text
print recovery_hint: focusa install --target=auto or focusa upgrade --dry-run
```

`focusa deck web` prints local URL and pairing instructions.

---

## 20. Implementation Architecture

### 20.1 TUI Changes

Likely files:

```text
crates/focusa-tui/src/main.rs
crates/focusa-tui/src/app.rs
crates/focusa-tui/src/api.rs
crates/focusa-tui/src/views/mod.rs
crates/focusa-tui/src/views/deck_home.rs          NEW
crates/focusa-tui/src/views/walkthrough.rs        NEW
crates/focusa-tui/src/views/mission_ladder.rs     NEW
crates/focusa-tui/src/views/recall.rs             NEW
crates/focusa-tui/src/views/release_readiness.rs  NEW
crates/focusa-tui/src/views/help_overlay.rs       NEW
```

### 20.2 CLI Changes

Likely files:

```text
crates/focusa-cli/src/commands/deck.rs  NEW
crates/focusa-cli/src/commands/mod.rs
crates/focusa-cli/src/main.rs
```

### 20.3 API Changes

Likely files:

```text
crates/focusa-api/src/routes/deck.rs  NEW
crates/focusa-api/src/routes/mod.rs
crates/focusa-api/src/server.rs
```

### 20.4 Web/PWA Changes

Preferred path:

```text
apps/deck/  NEW
```

Alternative, if launch speed requires reuse of existing Svelte/Tauri UI code:

```text
apps/menubar/src/lib/deck/*
```

Do not duplicate authority logic in the PWA. It consumes daemon/API envelopes.

### 20.5 Shared Schema

Add shared schema/types in Rust core or API layer:

```text
DeckHomePacket
DeckWalkthrough
DeckWalkthroughEvent
DeckMissionContractCard
DeckRecallCard
DeckReleaseReadinessCard
DeckFirstRunState
```

---

## 21. GTM Release Acceptance Gates

This spec is launch-relevant. It must not be marked done because the idea is written or because a mock exists.

### 21.1 First Encounter Gate

A clean install/eval user can run:

```bash
focusa start
focusa-tui
```

and see:

```text
Focusa Mission Deck name
beginner-friendly first screen
daemon state
project-bind next action
help/explain overlay
recovery hint if disconnected
```

### 21.2 Five-Minute Value Gate

A first-time evaluator can complete the First Mission walkthrough in five minutes or less on a compatible system.

Evidence must include:

```text
platform
install mode
commands run
screenshots or terminal capture
resulting project_root
continuity_id
Workpoint id or candidate id
evidence status
```

### 21.3 Recall Education Gate

Mission Deck includes a Recall tab or Recall command surface that clearly labels results as advisory and shows:

```text
scope
proof
freshness
memory status
allowed use
```

### 21.4 No-Proof-No-Done Gate

Mission Deck visibly distinguishes:

```text
claim without proof
linked proof
verified proof
```

### 21.5 Browser/PWA Gate

If PWA is included in the first launch cut, it must:

```text
serve /deck locally
include manifest and service worker
support pairing or local loopback access
be read-only by default
not expose raw terminal/shell by default
show revocation/security posture
```

If PWA is not included in the first launch cut, public claims must say it is planned, not shipped.

### 21.6 Install/Update Gate

Install architecture must remain respected:

```text
focusa-tui is an installed release asset
install path gives clear next command for Mission Deck
focusa upgrade --dry-run can identify stale installs
recovery hints point to focusa recover --dry-run, focusa doctor --scope host, and install/update commands
```

### 21.7 Claim-Discipline Gate

No task, bead, public page, release note, or final report may state Mission Deck/PWA/TUI onboarding is complete unless the required surface has actual evidence.

---

## 22. Implementation Phases

### Phase 0 — Spec and Decomposition

```text
Review this spec
Choose correct spec number / filename
Create parent bead
Create child beads
Add static guard ensuring the spec exists before implementation
```

### Phase 1 — Mission Deck Home in Native TUI

```text
Rename TUI title/header to Focusa Mission Deck
Add Deck Home tab/screen
Add Beginner Mode default decision tree
Add disconnected/unbound/no-workpoint/no-evidence/recovery cards
Add h explain overlay
Add n next safe action
```

### Phase 2 — Walkthrough Engine

```text
Add walkthrough schema
Add walkthrough store
Add First Mission walkthrough
Add Agent Handoff walkthrough
Add No Proof, No Done walkthrough
Expose walkthrough state via TUI and API
```

### Phase 3 — Visual Mission Ladder and Proof System

```text
Add Mission Ladder panel
Add Scope Badge
Add Proof Meter
Add Evidence Gap card
Add Drift Warning card
```

### Phase 4 — Mission Recall Tab

```text
Add / search behavior
Add RecallDeckCard schema
Add provider/session/source/scope/proof/freshness labels
Add memory status labels
Add Workpoint candidate promotion flow guarded by Context Authority
```

### Phase 5 — CLI Alias and Release Integration

```text
Add focusa deck alias/command
Add focusa deck web placeholder or implementation depending on PWA phase
Add install post-card mention: focusa-tui / focusa deck
Add upgrade/recover hints in Deck-disconnected states
```

### Phase 6 — Browser/PWA Mission Deck

```text
Add /deck static/PWA surface
Add manifest and service worker
Add mobile layout
Add QR/code pairing flow
Keep read-only default
Add revocation and security status
```

### Phase 7 — Optional Terminal Bridge

```text
Add terminal-in-browser only after PWA Mission Deck is safe
Start read-only
Gate write mode behind explicit operator approval
Audit every write-capable session
```

### Phase 8 — GTM Proof Package

```text
Capture first-run walkthrough proof
Capture screenshots/terminal recording
Capture install/update proof
Capture Recall advisory proof
Capture no-proof-no-done proof
Capture PWA proof if PWA is claimed
Update public docs and sales copy only for proven features
```

---

## 23. Bead Decomposition

Recommended parent:

```text
focusa-117-arch — EPIC: Implement Mission Deck onboarding, Recall, and PWA surface
```

Recommended children:

```text
focusa-117-spec-static-guard
focusa-117-tui-title-and-home
focusa-117-beginner-mode-state-machine
focusa-117-help-overlay
focusa-117-next-safe-action
focusa-117-walkthrough-schema
focusa-117-first-mission-walkthrough
focusa-117-agent-handoff-walkthrough
focusa-117-no-proof-no-done-walkthrough
focusa-117-mission-ladder-panel
focusa-117-proof-meter-and-scope-badge
focusa-117-recall-tab
focusa-117-recall-card-schema
focusa-117-workpoint-candidate-promotion
focusa-117-deck-cli-alias
focusa-117-deck-api-routes
focusa-117-pwa-static-shell
focusa-117-pwa-pairing
focusa-117-pwa-read-only-gate
focusa-117-terminal-bridge-readonly-design
focusa-117-release-install-postcard
focusa-117-gtm-five-minute-proof
focusa-117-public-docs-sync
```

Dependency order:

```text
spec-static-guard
→ tui-title-and-home
→ beginner-mode-state-machine
→ help-overlay + next-safe-action
→ walkthrough-schema
→ first-mission-walkthrough
→ visual panels
→ recall tab
→ CLI/API aliases
→ PWA
→ GTM proof/public docs
```

---

## 24. Required Tests and Proofs

### 24.1 Static Tests

Add:

```text
tests/spec117_mission_deck_static_test.sh
tests/spec117_walkthrough_schema_static_test.sh
tests/spec117_tui_beginner_mode_static_test.sh
tests/spec117_recall_card_authority_static_test.sh
tests/spec117_pwa_safe_surface_static_test.sh
tests/spec117_no_raw_terminal_default_static_test.sh
tests/spec117_deck_cli_alias_static_test.sh
```

### 24.2 Rust Checks

Required before closure:

```bash
cargo check -p focusa-tui -p focusa-cli -p focusa-api
cargo test -p focusa-tui -p focusa-cli -p focusa-api
```

### 24.3 Live Proof

At least one live proof must show:

```bash
focusa start
focusa-tui
```

and demonstrate:

```text
Mission Deck title
beginner-first card
next safe action
help overlay
project-bound Workpoint or explicit recovery path
evidence/proof state
Recall advisory labeling if Recall is in scope
```

### 24.4 PWA Proof

If PWA is claimed:

```text
/deck loads
manifest loads
service worker loads
mobile viewport works
pairing or loopback auth works
raw terminal is absent by default
mutation buttons are gated
revocation status is visible
```

---

## 25. Public Launch Copy Constraints

Allowed if proven:

```text
Focusa Mission Deck guides operators through project binding, Workpoints, proof, Recall, and safe resume.
```

Allowed if native TUI only is proven:

```text
Mission Deck is available as a terminal UI via focusa-tui.
```

Allowed if PWA is not implemented:

```text
Browser/PWA Mission Deck is planned.
```

Not allowed unless actual PWA proof exists:

```text
Focusa has a browser/PWA Mission Deck.
```

Not allowed unless terminal bridge proof exists:

```text
Focusa runs the TUI in the browser.
```

Not allowed unless write-gated terminal bridge proof exists:

```text
Focusa lets you control your terminal from the browser.
```

---

## 26. Security and Privacy Rules

```text
Mission Deck must not publish raw prompts/logs/diffs by default.
Recall excerpts must use redacted/safe projection.
PWA is read-only by default.
Terminal bridge is absent by default.
Pairing tokens are scoped and revocable.
Public cards require redaction and publish approval.
Remote access must not bypass local-first ownership.
Cloud coordinates; node decides.
Private state stays local unless explicitly published as a redacted receipt.
```

---

## 27. Completion Criteria

This implementation may be called complete only when:

```text
1. Correctly numbered spec exists in docs/.
2. Bead decomposition exists before implementation completion claims.
3. Native Mission Deck home exists in the TUI.
4. Beginner Mode renders disconnected, unbound, no-Workpoint, no-evidence, and resumable states.
5. First Mission walkthrough exists and is repeatable.
6. Help overlay explains Workpoint, Evidence, Recall, Mission Ladder, and authority badges.
7. Mission Ladder and Proof Meter render current state or explicit unavailable/recovery states.
8. Recall results are labeled advisory and include scope/proof/freshness/memory status.
9. Mutating actions remain gated by Context Authority or are visibly not implemented.
10. focusa deck alias exists or public docs explicitly say focusa-tui is the current command.
11. PWA claims are either proven or omitted from launch copy.
12. Static tests exist and pass.
13. Cargo checks/tests pass for touched crates.
14. Live first-run proof exists.
15. Final report cites actual evidence and names any partial/surrogate/blocked evidence.
```

---

## 28. Release / GTM Decision Rule

For first launch/release, prioritize in this order:

```text
1. Native Mission Deck first encounter
2. Beginner Mode and First Mission walkthrough
3. Proof/no-proof education
4. Recall advisory labeling
5. focusa deck alias and install postcard
6. Browser/PWA Mission Deck
7. Terminal-in-browser bridge
```

The first launch must **not** wait for browser terminal support.

The first launch **should** wait for a strong native Mission Deck first encounter, because that is the buyer’s fastest path to understanding Focusa.

---

## 29. Summary

Focusa already has the runtime, CLI, daemon, TUI, Workpoints, Evidence, Context Authority, and install path.

This spec makes the launch experience easier to understand, easier to repeat, and easier to prove.

Mission Deck is not a cosmetic dashboard. It is the guided operator surface that turns Focusa from:

```text
powerful infrastructure
```

into:

```text
I understand why I need this.
```