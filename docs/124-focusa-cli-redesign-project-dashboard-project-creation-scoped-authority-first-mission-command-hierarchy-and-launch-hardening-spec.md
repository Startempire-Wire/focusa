# Spec 124 — Complete Focusa CLI Redesign, Project Dashboard, Project Creation, Scoped Authority, First Mission, Command Hierarchy, and Launch Hardening

**Status:** Complete implementation spec
**Priority:** Complete pre-launch implementation
**Scope:** Focusa CLI, project dashboard, project discovery, project creation, project registry/profile UX, project templates, per-project settings, selected-project observability, active-project observability, scoped command resolver, scope-safety expansion, first-mission walkthrough, status redesign, help redesign, pairing consolidation, command hierarchy cleanup, launch hardening fixes, TUI/API route fixes, uninstall/cleanup/stop fixes, docs, tests, aliases, migration help, and anti-singleton architecture enforcement.
**Rule:** Phases below are sequencing only. Nothing in this spec is deferred, optional, or post-launch.
**Boundary with Spec 123:** This spec owns the CLI/operator workflow; Spec 123 owns public-safe repo presentation, docs sanitation, and private boundary layout.

---

## 0. Executive summary

Focusa’s CLI must become a complete, interactive, project-aware operator experience.

The central command becomes:

```bash
focusa project
```

This opens a project dashboard where users can:

```text
- see recent/known projects
- see the selected CLI project
- view daemon-observed active project/workstream surfaces
- bind/select an existing project
- discover nearby Git/project roots
- create a new blank Focusa project
- initialize Focusa root configuration
- manage project settings
- choose project templates
- run First Mission
```

The launch proof command becomes:

```bash
focusa first-mission
```

This guides the evaluator through:

```text
choose or create project
→ verify scope
→ start daemon
→ create Workpoint
→ attach proof
→ render resume packet
→ open Mission Deck
```

The local-agent feedback correctly identifies top-level CLI sprawl, pairing-command duplication, mixed singleton/scoped command behavior, inconsistent scope enforcement, missing project navigation, and the need for a central `focusa project` surface.

The non-negotiable architectural rule:

```text
Focusa supports active-project observability.
Focusa must not use hidden active-project authority.
```

---

## 1. Current state and problem

### 1.1 CLI is powerful but too flat

The CLI currently exposes a large top-level surface. Users see many advanced commands before they understand the basic workflow:

```text
start
stop
about
install
upgrade
uninstall
status
onboard
pair
pairing
pairing-transport
pairing-doctor
pairing-wizard
doctor
cleanup
continue
tui
init
walkthrough
deck
workflow
focus
stack
gate
workpoint
trajectory
hlt
context-cognition
memory
turns
audit
lineage
clt
constitution
telemetry
rfm
predict
reflect
metacognition
ontology
skills
thread
export
contribute
cache
project
resource
tokens
wrap
...
```

The local agent observed that the CLI exposes 60+ commands at the top level and that users cannot predict where a command lives without reading docs or guessing.

### 1.2 Pairing command sprawl is real

The current CLI exposes multiple pairing entry points for related behavior:

```text
focusa pair
focusa pairing
focusa pairing-transport
focusa pairing-doctor
focusa pairing-wizard
```

The local agent identified these as duplicate or overlapping pairing surfaces.

### 1.3 Project navigation is missing

Current project-related CLI behavior can inspect or verify a project when the user already knows what to pass, but it does not provide a complete project navigation UX.

The local agent observed that every scoped command requires `--project-root <path>` and there is no concept of “I am working in project X now.”

The CLI needs:

```text
focusa project
focusa project list
focusa project discover
focusa project use
focusa project bind
focusa project switch
focusa project current
focusa project status
focusa project new
focusa project settings
focusa project templates
```

### 1.4 Scope safety is not consistently applied

The local agent observed that the shared `classify_project_root()` scope-safety classifier is used by some surfaces, but not all. It specifically called out that `trajectory`, `workpoint`, `project`, and `recover` use safety checks, while `focus`, `hlt`, `context-cognition`, `memory`, `turns`, `audit`, and `cleanup` need additional enforcement.

This spec requires scope enforcement expansion across all scoped or potentially scoped command families.

### 1.5 Current CLI lacks complete first-run capture

A new user needs one obvious path from install to value:

```bash
focusa first-mission
```

This must prove:

```text
- project identity
- scoped authority
- Workpoint
- proof/evidence
- resume packet
- next safe action
- Mission Deck handoff
```

### 1.6 Active-project observability is good; hidden authority is not

Focusa was built for observability. Users must be able to inspect current active project/workstream state.

The problem is not showing an active project. The problem is silently mutating hidden active state without explicit verified scope.

---

## 2. Non-negotiable architecture rule

### 2.1 Selected CLI project is convenience only

Focusa may store a selected CLI project.

It must not become hidden canonical authority.

Allowed:

```text
- selected CLI project profile
- recent projects
- current selected project display
- daemon-observed active project/workstream display
- project dashboard
- project status card
- command defaults populated from selected project
```

Required:

```text
- explicit project_root passed into canonical API calls
- continuity_id passed where required
- scope_source shown in output
- authority status shown in output
- scope safety check before project mutation/resume/checkpoint
```

Forbidden:

```text
- unscoped mutation because an active project exists
- daemon-global current project as authority
- continuity_id-only authority
- hidden fallback from unsafe cwd to prior selected project
- host scope masquerading as project scope
- unscoped daemon /v1/project/identity as source of project authority
```

### 2.2 Active-project observability is required

The user must be able to inspect current state.

Commands must expose:

```bash
focusa project current
focusa project status
focusa status operator
focusa workpoint current
focusa trajectory view
focusa focus current
```

These commands must distinguish:

```text
Selected CLI project:
  local CLI convenience profile

Runtime observed project/workstream:
  what the daemon reports from scoped state

Canonical authority scope:
  explicit verified project/host scope used for mutation/checkpoint/resume/proof
```

The local agent originally proposed an “active project” file and resolution chain. The UX need is correct, but the authority semantics must be corrected: the selected CLI project is a convenience pointer only.

### 2.3 Better UX must not smuggle singleton authority back into Focusa

The CLI may become:

```text
- more navigable
- more hierarchical
- more interactive
- more helpful
- more project-aware
```

It must not become:

```text
- less explicit about scope
- more dependent on hidden active state
- silently fallback-driven
- singleton-shaped
```

Final rule:

```text
Focusa may remember what the operator selected.
Focusa may show what is active.
Focusa must verify before it mutates.
```

---

## 3. Complete command hierarchy target

### 3.1 Target hierarchy

```text
focusa
├── project                 # project dashboard, recent projects, creation, discovery
├── first-mission           # guided evaluator workflow
├── status                  # agent/operator status cards
├── deck                    # Mission Deck launcher / alias to TUI
├── doctor                  # direct diagnostic shortcut
├── menu                    # general interactive CLI menu
├── pairing                 # all local/phone pairing
├── setup                   # wizard/init/doctor/walkthrough
├── lifecycle               # start/stop/install/uninstall/upgrade/service/codesign
├── work                    # continue/workflow/work-item/threads/recover
├── focus                   # focus state + stack
├── workpoint               # checkpoint/current/resume/evidence/drift
├── trajectory              # view/define/assess/propose/checkpoint/resume
├── hlt                     # high-level trajectory ledger
├── evidence                # proposals/predict/reflect/metacognition/ecs
├── runtime                 # telemetry/cache/autonomy/rfm/resource/tokens
├── agent                   # skills/awareness/ontology/preload/bloatgaurd/constitution
├── state                   # state dump/snapshot/events/debug
├── cleanup                 # safe cleanup/dry-run
├── explain                 # explain failure/recovery
├── quality                 # release/dxux/utility/explain
├── export                  # dataset export
├── contribute              # data contribution
├── wrap                    # harness CLI wrapper
└── help                    # curated/help-all/per-command help
```

### 3.2 Backward compatibility

Old commands continue to work with warnings during the deprecation window.

Examples:

```text
focusa init                  → focusa project new / focusa setup init
focusa onboard               → focusa setup wizard
focusa stack                 → focusa focus stack
focusa pair                  → focusa pairing start
focusa pairing-doctor        → focusa pairing doctor
focusa pairing-transport     → focusa pairing transport
focusa pairing-wizard        → focusa pairing wizard
focusa status --agent        → focusa status agent
focusa status --operator     → focusa status operator
```

Warnings must be explicit:

```text
Deprecated command.

Use:
  focusa status operator

This alias will become an error after:
  <date>

See:
  focusa help migration
```

No old command may silently behave differently.

---

## 4. Command placement decisions

### 4.1 `constitution`

Move under `agent`, preserve old top-level alias.

Canonical:

```bash
focusa agent constitution active
focusa agent constitution versions
```

Alias:

```bash
focusa constitution active
focusa constitution versions
```

Reason:

```text
Constitution is an agent behavior/governance surface.
```

### 4.2 `threads`

Move under `work`.

Canonical:

```bash
focusa work threads list
focusa work threads create
focusa work threads get
focusa work threads fork
focusa work threads transfer
```

Alias:

```bash
focusa threads list
focusa threads create
focusa threads get
focusa threads fork
focusa threads transfer
```

Reason:

```text
Threads are execution/work-continuity surfaces.
```

### 4.3 `tokens`

Move under `runtime`.

Canonical:

```bash
focusa runtime tokens create
focusa runtime tokens revoke
focusa runtime tokens list
focusa runtime tokens doctor
focusa runtime tokens compact-plan
```

Alias:

```bash
focusa tokens create
focusa tokens revoke
focusa tokens list
focusa tokens doctor
focusa tokens compact-plan
```

Reason:

```text
Tokens are runtime access/control infrastructure.
```

### 4.4 `ecs`

Move under `evidence`, preserve old top-level alias.

Canonical:

```bash
focusa evidence ecs list
focusa evidence ecs resolve
```

Alias:

```bash
focusa ecs list
focusa ecs resolve
```

Reason:

```text
ECS/reference store is evidence/reference substrate.
```

### 4.5 `call-stack`

Keep the existing name. Do not introduce `calls`.

Canonical:

```bash
focusa call-stack design
focusa call-stack verify
focusa call-stack list
focusa call-stack show
```

Reason:

```text
The current binary and docs use call-stack. Do not create avoidable naming drift.
```

### 4.6 `codesign`

Move under `lifecycle`, preserve old alias.

Canonical:

```bash
focusa lifecycle codesign
```

Alias:

```bash
focusa codesign
```

Reason:

```text
Codesign is release/install/lifecycle support.
```

### 4.7 `recover`

Move under `work`, preserve top-level alias.

Canonical:

```bash
focusa work recover
```

Alias:

```bash
focusa recover
```

Reason:

```text
Recover is a work-continuity command.
```

### 4.8 `walkthrough`

Move under `setup`, preserve top-level alias.

Canonical:

```bash
focusa setup walkthrough list
focusa setup walkthrough show
focusa setup walkthrough run
```

Alias:

```bash
focusa walkthrough list
focusa walkthrough show
focusa walkthrough run
```

Reason:

```text
Walkthroughs are setup/education flows.
```

### 4.9 `explain`

Keep top-level and also expose under `quality`.

Canonical top-level:

```bash
focusa explain <failure>
```

Secondary:

```bash
focusa quality explain <failure>
```

Reason:

```text
Explain is useful enough as a recovery primitive to stay top-level.
```

### 4.10 `wrap`, `export`, `contribute`

Keep top-level.

```bash
focusa wrap -- <command>
focusa export ...
focusa contribute ...
```

Reason:

```text
These are specialized but clear.
```

### 4.11 `deck` and `tui`

`focusa deck` is the user-facing Mission Deck launcher.

```bash
focusa deck
```

`focusa tui` remains a lower-level technical TUI command.

```bash
focusa tui
focusa tui --headless-self-test
```

`focusa deck --help` should describe the product surface:

```text
Open Focusa Mission Deck — the local terminal cockpit for project, Workpoint, proof, trajectory, recall, and next safe action.
```

`focusa tui --help` can remain technical.

---

## 5. `focusa project` interactive dashboard

### 5.1 Command

```bash
focusa project
```

### 5.2 Interactive output

```text
FOCUSA PROJECTS

Selected CLI project:
  Focusa
  /Users/verious/code/focusa

Runtime observed:
  Workpoint: active
  Trajectory: provisional
  Proof: 2 refs
  Health: healthy

Recent projects:
  1. Focusa          /Users/verious/code/focusa
  2. UIAI Engine     /Users/verious/code/uiai-engine
  3. StartEmpire     /Users/verious/code/startempire-wire

Options:
  [1-3] Use project
  d     Discover projects nearby
  b     Bind existing project by path
  n     New blank project
  c     Show current project
  s     Project settings
  t     Templates
  f     Run First Mission
  q     Quit
```

### 5.3 Non-interactive output

```bash
focusa project --json
```

```json
{
  "schema": "focusa.project_dashboard.v1",
  "status": "completed",
  "selected_project": {
    "project_id": "focusa",
    "project_root": "/Users/verious/code/focusa",
    "scope_source": "cli_profile",
    "authority": "convenience_only"
  },
  "runtime_observed": {
    "project_root": "/Users/verious/code/focusa",
    "workpoint_status": "active",
    "trajectory_status": "provisional",
    "health": "healthy"
  },
  "recent_projects": []
}
```

---

## 6. Project commands

### 6.1 `focusa project list`

Lists known projects.

```bash
focusa project list
focusa project list --json
```

Human output:

```text
Known Focusa projects:

* focusa
  /home/wirebot/focusa
  selected · verified 2m ago

  uiai-engine
  /home/wirebot/uiai-engine
  not selected · last verified yesterday
```

### 6.2 `focusa project discover`

Finds nearby safe project roots.

```bash
focusa project discover
focusa project discover --from /home/wirebot
focusa project discover --max-depth 3
focusa project discover --json
```

Discovery must be bounded:

```text
- no full-home recursive scan by default
- max depth default: 2
- max candidate dirs default: 500
- ignore node_modules, target, vendor, .cache, .local/share, .git internals
- never scan /proc, /sys, /dev, /run
- broad roots are never candidates
```

Candidate scoring:

```text
+50 .git present
+35 git rev-parse confirms root
+25 .focusa-project.json present
+20 Cargo.toml / package.json / go.mod / pyproject.toml
+15 README present
+10 src/ crates/ apps/ present
+10 cwd is inside this repo
-100 unsafe broad root
-100 unsafe user home
-100 agent runtime directory
-30 node_modules / target / vendor
```

The local agent explicitly called out project auto-detection and bounded resolver behavior as important to the project UX.

### 6.3 `focusa project use`

Selects a known project for CLI convenience.

```bash
focusa project use focusa
focusa project use /home/wirebot/focusa
focusa project use 1
```

Output:

```text
Selected CLI project:
  focusa
  /home/wirebot/focusa

This is a local CLI convenience profile.
Canonical authority still requires scoped API verification.
```

### 6.4 `focusa project bind`

Adds and selects an existing project/repo by path.

```bash
focusa project bind /Users/verious/code/focusa
focusa project bind
```

If no marker exists:

```text
No Focusa project marker found.

Create .focusa-project.json here?
  /Users/verious/code/focusa

[Y/n]
```

### 6.5 `focusa project switch`

Interactive recent-project switcher.

```bash
focusa project switch
```

### 6.6 `focusa project current`

Shows selected CLI project and runtime observed state.

```bash
focusa project current
focusa project current --json
```

Output:

```text
Selected CLI project:
  focusa
  /home/wirebot/focusa

Scope:
  safe project root

Verification:
  marker present
  git root verified
  ProjectIdentity confidence high

Runtime observed:
  Workpoint: active
  Trajectory: provisional
  Focus: available
  Proof: linked

Next:
  focusa status operator
```

### 6.7 `focusa project status`

Project-specific card.

```bash
focusa project status
focusa project status --project focusa
focusa project status --project-root /path/to/repo
focusa project status --json
```

Must show:

```text
- selected CLI project
- project identity
- runtime observed active surfaces
- Workpoint
- Trajectory
- Focus
- proof/evidence
- next safe action
- scope_source
- authority status
```

### 6.8 `focusa project remove`

Removes local project profile only.

```bash
focusa project remove focusa
focusa project remove --all
```

Must not delete project files.

---

## 7. New blank project creation

### 7.1 Command

```bash
focusa project new
focusa project new --working-dir ~/Code
focusa project new --name my-new-project
focusa project new --working-dir ~/Code --name my-new-project --git
focusa project new --template blank
```

### 7.2 Interactive prompt

```text
New Focusa project

Working directory:
  ~/Code

Project name:
  my-new-project

Template:
  blank

Create Git repo? [Y/n]
Create .focusa-project.json? [Y/n]
Create .focusa/ directory? [Y/n]
Use as selected CLI project? [Y/n]
Run First Mission after creation? [Y/n]
```

### 7.3 Created structure

```text
/path/to/workdir/my-new-project/
├── .focusa-project.json
├── .focusa/
│   ├── settings.json
│   ├── evidence/
│   ├── workpoints/
│   ├── trajectories/
│   ├── templates/
│   └── README.md
├── README.md
└── .git/                 # optional
```

### 7.4 `.focusa-project.json`

```json
{
  "schema": "focusa.project.v1",
  "project_id": "my-new-project",
  "canonical_name": "My New Project",
  "project_root": "/path/to/workdir/my-new-project",
  "beads_prefix": "my-new-project",
  "workspace_kind": "blank",
  "aliases": [],
  "created_at": "2026-07-08T00:00:00Z"
}
```

### 7.5 `.focusa/settings.json`

```json
{
  "schema": "focusa.project_settings.v1",
  "project_id": "my-new-project",
  "proof_policy": "proof_or_explicit_gap",
  "default_continuity_id": "my-new-project-main",
  "created_by": "focusa project new",
  "authority": "local_project_preferences_only"
}
```

### 7.6 Safety rules

```text
1. normalize working directory
2. reject unsafe project root
3. reject project name traversal: ../, /, ~ inside name
4. require final path under chosen working directory
5. fail if final path exists and is non-empty unless explicit force flag is used
6. never overwrite existing .focusa-project.json without confirmation
7. never treat .focusa/settings.json as canonical daemon authority
```

---

## 8. Project templates

Templates are included in the complete spec.

### 8.1 Commands

```bash
focusa project templates list
focusa project templates show blank
focusa project templates show web-app
focusa project new --template blank
focusa project new --template web-app
```

### 8.2 Required templates

```text
blank
web-app
cli-tool
rust-service
node-saas
wordpress-plugin
agent-workbench
```

### 8.3 Template location

Built-in templates:

```text
crates/focusa-cli/templates/project/
```

User templates:

```text
~/.config/focusa/project-templates/
```

### 8.4 Template metadata

Each template has:

```json
{
  "schema": "focusa.project_template.v1",
  "name": "web-app",
  "description": "Basic web application Focusa project",
  "files": [],
  "directories": [],
  "post_create_hints": []
}
```

---

## 9. Project registry and selected CLI profile

### 9.1 Storage

```text
~/.config/focusa/
├── selected-project.json
├── projects/
│   └── <project_fingerprint>.json
├── project-settings/
│   └── <project_fingerprint>.json
└── project-templates/
```

Compatibility alias:

```text
~/.config/focusa/active-project
```

If this exists, migrate it to `selected-project.json`.

### 9.2 Selected project schema

```json
{
  "schema": "focusa.cli.selected_project.v1",
  "selected_project_fingerprint": "project-fnv1a64:...",
  "selected_at": "2026-07-08T00:00:00Z",
  "selected_by": "focusa project use",
  "note": "CLI convenience profile only; not canonical daemon authority"
}
```

### 9.3 Project profile schema

```json
{
  "schema": "focusa.cli.project_profile.v1",
  "project_id": "focusa",
  "canonical_name": "Focusa",
  "aliases": ["focusa"],
  "project_root": "/home/wirebot/focusa",
  "fingerprint": "project-fnv1a64:...",
  "marker_path": "/home/wirebot/focusa/.focusa-project.json",
  "repo_remote": "https://github.com/Startempire-Wire/focusa.git",
  "workspace_kind": "rust-monorepo",
  "scope_safety": "safe",
  "last_verified_at": "2026-07-08T00:00:00Z",
  "created_at": "2026-07-08T00:00:00Z"
}
```

---

## 10. Project settings

### 10.1 Commands

```bash
focusa project settings list
focusa project settings get default_continuity_id
focusa project settings set default_continuity_id focusa-main
focusa project settings unset default_continuity_id
```

### 10.2 Settings

```text
default_continuity_id
default_agent
default_workflow
open_deck_after_first_mission
proof_policy
preferred_editor
preferred_evidence_paths
default_template
first_mission_auto_open_deck
status_card_detail_level
```

Settings are preferences only.

---

## 11. Scope resolver

### 11.1 New module

```text
crates/focusa-cli/src/commands/scope_resolver.rs
```

### 11.2 Resolution order

```text
1. explicit --project-root
2. explicit --project <alias/id>
3. FOCUSA_PROJECT_ROOT
4. FOCUSA_SELECTED_PROJECT
5. FOCUSA_ACTIVE_PROJECT legacy alias
6. selected CLI project profile
7. cwd upward .focusa-project.json
8. cwd git root if safe
9. interactive discovery/picker when TTY
10. blocked envelope
```

Rejected:

```text
GET /v1/project/identity with no project_root as fallback
```

Allowed:

```text
candidate root → API verifies candidate root
```

Not allowed:

```text
no candidate root → daemon supplies last project
```

### 11.3 Resolver output

```rust
pub struct ResolvedProjectScope {
    pub project_root: String,
    pub continuity_id: Option<String>,
    pub project_id: Option<String>,
    pub fingerprint: Option<String>,
    pub scope_source: ScopeSource,
    pub verified: bool,
}

pub enum ScopeSource {
    ExplicitFlag,
    ProjectAlias,
    EnvProjectRoot,
    EnvSelectedProject,
    LegacyEnvActiveProject,
    CliSelectedProject,
    CwdMarker,
    CwdGitRoot,
    InteractiveSelection,
}
```

### 11.4 Blocked envelope

If no project is resolved:

```json
{
  "status": "blocked",
  "failure_class": "project_root_selection_required",
  "next_step_hint": "Run focusa project, focusa project discover, or pass --project-root <path>."
}
```

---

## 12. Scope safety expansion

Every command below must use the resolver and/or `ensure_project_root_scope_safe()`:

```text
focus
hlt
context-cognition
memory
turns
audit
cleanup
lineage
clt
call-stack
gate
workpoint
trajectory
project
recover
```

Commands without API support for scoping must clearly output:

```json
{
  "authority": "daemon_global_advisory",
  "canonical": false,
  "next_step_hint": "This surface needs Spec104 scoped API work before it can be treated as project-canonical."
}
```

No command may pretend to be project-scoped when the API remains daemon-global.

---

## 13. Status redesign

### 13.1 Commands

```bash
focusa status agent
focusa status operator
```

Aliases:

```text
focusa status --agent
focusa status --operator
```

### 13.2 Operator status must be scoped

Required API shape:

```text
GET /v1/project/identity?project_root=<resolved>
GET /v1/trajectory/view?project_root=<resolved>&continuity_id=<resolved>&mode=summary
POST /v1/workpoint/resume
  {
    "project_root": "<resolved>",
    "continuity_id": "<resolved>",
    "mode": "operator_summary"
  }
```

Unscoped operator status is forbidden.

### 13.3 Output

```text
FOCUSA OPERATOR CARD

Selected CLI project:
  focusa
  /home/wirebot/focusa

Runtime observed:
  Workpoint: active
  Trajectory: provisional
  Focus: available

Canonical authority scope:
  project_root: /home/wirebot/focusa
  continuity_id: focusa-main
  scope_source: cli_profile
  authority: verified

Proof:
  2 refs linked

Next:
  focusa workpoint resume --project focusa
```

---

## 14. First Mission

### 14.1 Command

```bash
focusa first-mission
focusa first-mission --project-root /path/to/repo
focusa first-mission --project focusa
focusa first-mission --continuity-id focusa-main
focusa first-mission --yes
focusa first-mission --dry-run
focusa first-mission --json
focusa first-mission --open-deck
focusa first-mission --no-animation
```

### 14.2 Flow

```text
1. resolve/select/create project
2. verify scope safety
3. start daemon
4. init marker if missing
5. create Workpoint
6. attach proof
7. render resume packet
8. show project status
9. suggest/open Mission Deck
```

### 14.3 Output

```text
FOCUSA FIRST MISSION

Give this AI project a save point, proof, and safe handoff.

✓ Project selected: /home/wirebot/focusa
✓ Scope safe
✓ Daemon healthy
✓ Project marker present
✓ Workpoint created
✓ Proof linked
✓ Resume packet ready

Mission saved.

Next:
  focusa deck
  focusa status operator
  focusa workpoint resume --project focusa
```

---

## 15. Focus namespace

### 15.1 Commands

```bash
focusa focus
focusa focus current
focusa focus stack
focusa focus push
focusa focus pop
focusa focus update
```

### 15.2 Default behavior

Running:

```bash
focusa focus
```

with no subcommand should default to:

```bash
focusa focus current
```

not help text.

### 15.3 Required output

```text
FOCUSA FOCUS

Current frame:
  <frame title or none>

Current focus:
  <current focus summary>

Stack:
  active frame: <id>
  frames: <count>

Next:
  focusa focus update --next-step "..."
  focusa focus stack
```

### 15.4 Stack alias

```bash
focusa stack
```

must warn and run:

```bash
focusa focus stack
```

Warning:

```text
Deprecated alias. Use: focusa focus stack
```

---

## 16. Setup namespace

### 16.1 Commands

```bash
focusa setup wizard
focusa setup init
focusa setup doctor
focusa setup walkthrough list
focusa setup walkthrough show
focusa setup walkthrough run
```

Aliases:

```text
focusa onboard     → focusa setup wizard
focusa init        → focusa setup init
focusa preflight   → focusa setup doctor or focusa quality preflight
focusa walkthrough → focusa setup walkthrough
```

### 16.2 `setup wizard`

Uses project dashboard and discovery:

```text
choose project or host scope
→ discover/bind/create
→ verify
→ start daemon
→ optional first mission
```

---

## 17. Lifecycle namespace

### 17.1 Commands

```bash
focusa lifecycle start
focusa lifecycle stop
focusa lifecycle install
focusa lifecycle uninstall
focusa lifecycle upgrade
focusa lifecycle install-service
focusa lifecycle codesign
focusa lifecycle doctor
```

Aliases:

```text
focusa start
focusa stop
focusa install
focusa uninstall
focusa upgrade
focusa install-service
focusa codesign
```

### 17.2 Required hardening

```text
- fix focusa stop contradictory output
- implement uninstall --keep-data
- implement uninstall --keep-license
- implement uninstall --keep-path-modifications
- implement uninstall --purge
- or remove nonfunctional flags from help
```

---

## 18. Pairing consolidation

### 18.1 Canonical commands

```bash
focusa pairing start
focusa pairing wizard
focusa pairing create-room
focusa pairing transport
focusa pairing doctor
focusa pairing status
focusa pairing history
focusa pairing email-link
focusa pairing cycle-test
```

### 18.2 Aliases

```text
focusa pair                → focusa pairing start
focusa pairing-transport   → focusa pairing transport
focusa pairing-doctor      → focusa pairing doctor
focusa pairing-wizard      → focusa pairing wizard
```

---

## 19. Runtime namespace

### 19.1 Commands

```bash
focusa runtime telemetry tokens
focusa runtime telemetry token-budget
focusa runtime telemetry cost
focusa runtime cache doctor
focusa runtime cache status
focusa runtime autonomy status
focusa runtime rfm status
focusa runtime resource status
focusa runtime resource activate-lowmem
focusa runtime resource deactivate-lowmem
focusa runtime tokens create
focusa runtime tokens revoke
focusa runtime tokens list
focusa runtime tokens doctor
focusa runtime tokens compact-plan
```

Aliases remain where currently available.

---

## 20. Evidence namespace

### 20.1 Commands

```bash
focusa evidence proposals list
focusa evidence proposals submit
focusa evidence proposals resolve
focusa evidence predict record
focusa evidence predict evaluate
focusa evidence predict capture-outcome
focusa evidence predict recent
focusa evidence predict stats
focusa evidence reflect run
focusa evidence reflect history
focusa evidence metacognition capture
focusa evidence metacognition retrieve
focusa evidence metacognition reflect
focusa evidence metacognition adjust
focusa evidence metacognition evaluate
focusa evidence ecs list
focusa evidence ecs resolve
```

Existing top-level `predict`, `reflect`, `metacognition`, and `ecs` commands remain as aliases with warnings.

---

## 21. Agent namespace

### 21.1 Commands

```bash
focusa agent skills list
focusa agent awareness card
focusa agent ontology primitives
focusa agent ontology world
focusa agent ontology contracts
focusa agent preload profiles
focusa agent bloatgaurd report
focusa agent bloatgaurd domain
focusa agent bloatgaurd tokenbloat
focusa agent bloatgaurd token-domain
focusa agent bloatgaurd gate-modes
focusa agent constitution active
focusa agent constitution versions
```

Aliases remain.

---

## 22. Work namespace

### 22.1 Commands

```bash
focusa work continue
focusa work workflow list
focusa work workflow show
focusa work item propose
focusa work item list
focusa work item close
focusa work item verify
focusa work threads list
focusa work threads create
focusa work threads get
focusa work threads fork
focusa work threads transfer
focusa work recover
```

Aliases:

```text
focusa continue
focusa workflow
focusa work-item
focusa threads
focusa recover
```

---

## 23. Cleanup, recover, explain, export, contribute, wrap

### 23.1 Cleanup

```bash
focusa cleanup safe
focusa cleanup dry-run
```

Required:

```text
- fix cleanup --safe self-blocking
- cleanup must be scoped or clearly daemon-global advisory
```

### 23.2 Explain

```bash
focusa explain <failure>
focusa quality explain <failure>
```

### 23.3 Export / contribute / wrap

Keep top-level:

```bash
focusa export ...
focusa contribute ...
focusa wrap -- <command>
```

---

## 24. Help redesign

### 24.1 `focusa -h`

```text
FOCUSA
Mission continuity for AI coding agents.
Save the mission. Prove the work. Resume safely.

Start here:
  focusa project             Open project dashboard
  focusa first-mission       Guided evaluator workflow
  focusa project discover    Find projects Focusa can bind
  focusa deck                Open Mission Deck

Project:
  focusa project list
  focusa project use <name>
  focusa project switch
  focusa project new
  focusa project current
  focusa project status

Daily:
  focusa status operator
  focusa workpoint resume
  focusa doctor

Advanced:
  focusa help all
  focusa <command> --help
```

### 24.2 Full help

```bash
focusa help all
focusa help project
focusa help workpoint
focusa help migration
```

### 24.3 Migration help

`focusa help migration` must show old → new commands and deprecation dates.

---

## 25. Deprecation timeline

### 25.1 Rule

Deprecated aliases warn for **90 days after the canonical replacement ships in a tagged release**, then become hard errors.

### 25.2 Warning format

```text
Deprecated command.

Use:
  focusa status operator

This alias will become an error after:
  2026-10-08

See:
  focusa help migration
```

### 25.3 Hard-error format

```text
This command has been removed.

Use:
  focusa status operator
```

### 25.4 Exceptions

Security, data-loss, or authority-risky aliases may become hard errors sooner.

Examples:

```text
- unscoped old operator-status paths
- unsafe root bypass shortcuts
- commands that silently mutate singleton/global state
```

---

## 26. Launch hardening fixes

These are included in the complete implementation.

```text
1. TUI/API route mismatches
   - fix GET/POST mismatch for Workpoint resume
   - fix telemetry snapshot route or update TUI consumer

2. Doctor path resolution
   - fix daemon_exe_path / blocked check

3. Context cognition scope fallback
   - remove unsafe fallback
   - require explicit or resolver-provided verified scope

4. Cleanup self-blocking
   - fix cleanup --safe

5. Stop command output
   - stopped / already stopped / failed must be distinct

6. Uninstall keep flags
   - wire --keep-data
   - wire --keep-license
   - wire --keep-path-modifications
   - wire --purge

7. Pairing room cleanup
   - verify expired pairing rooms clean at startup

8. FOCUSA_NO_DECAY_TICK
   - document if supported
   - remove if obsolete
```

---

## 27. Implementation phases

Phases are sequencing only. The spec is not complete until all phases are complete.

### Phase 1 — Hardening fixes

```text
- TUI/API route mismatch fixes
- doctor blocked-check fix
- context-cognition scope fallback fix
- cleanup --safe fix
- focusa stop output fix
- uninstall --keep-* implementation
- pairing room cleanup verification
- FOCUSA_NO_DECAY_TICK docs/removal
```

### Phase 2 — Project surface

```text
- focusa project dashboard
- project list/discover/use/bind/switch/current/status/remove
- project new
- project templates
- project settings
- selected-project.json
- project profile registry
```

### Phase 3 — Scope resolver and enforcement

```text
- scope_resolver.rs
- resolver used by all scoped commands
- expand scope safety
- block/advisory classification for still-global surfaces
```

### Phase 4 — Status/help/pairing

```text
- status agent/operator
- status operator scoped
- curated focusa -h
- focusa help all
- focusa help migration
- pairing consolidation
```

### Phase 5 — First Mission

```text
- focusa first-mission
- project selection/create path
- Workpoint creation
- proof link
- resume packet
- deck handoff
```

### Phase 6 — Namespace completion

```text
- setup namespace
- lifecycle namespace
- runtime namespace
- evidence namespace
- agent namespace
- work namespace
- aliases with warnings
- docs update
```

No phase is optional.

---

## 28. Cross-phase integration smoke tests

After every implementation phase, run the same integration smoke suite.

### 28.1 Smoke test command set

```bash
focusa project list
focusa project current --json || true
focusa project discover --max-depth 2 --json
focusa status operator --json
focusa first-mission --dry-run --json
focusa deck --headless-self-test || focusa tui --headless-self-test
```

### 28.2 Alias smoke tests

```bash
focusa status --operator --json
focusa status operator --json
focusa stack
focusa focus stack
focusa pair --help
focusa pairing start --help
```

### 28.3 Required assertions

The smoke test must prove:

```text
- canonical commands work
- old aliases still route or warn correctly
- status operator is scoped or blocks with project_root_selection_required
- first-mission dry-run works without mutation
- project discovery does not select unsafe roots
- deck/tui self-test still works
```

### 28.4 Test file

```text
tests/spec_cli_cross_phase_smoke_test.sh
```

This test must run in CI after every phase branch/PR.

---

## 29. Files to modify

### 29.1 New files

```text
crates/focusa-cli/src/commands/scope_resolver.rs
crates/focusa-cli/src/commands/project_registry.rs
crates/focusa-cli/src/commands/project_discovery.rs
crates/focusa-cli/src/commands/project_new.rs
crates/focusa-cli/src/commands/project_templates.rs
crates/focusa-cli/src/commands/project_settings.rs
crates/focusa-cli/src/commands/first_mission.rs
crates/focusa-cli/src/commands/menu.rs
crates/focusa-cli/src/commands/setup.rs
crates/focusa-cli/src/commands/lifecycle.rs
crates/focusa-cli/src/commands/status.rs
crates/focusa-cli/src/commands/migration_help.rs
tests/spec_cli_cross_phase_smoke_test.sh
```

### 29.2 Modified files

```text
crates/focusa-cli/src/main.rs
crates/focusa-cli/src/commands/mod.rs
crates/focusa-cli/src/commands/intro.rs
crates/focusa-cli/src/commands/project.rs
crates/focusa-cli/src/commands/scope.rs
crates/focusa-cli/src/commands/onboard.rs
crates/focusa-cli/src/commands/init.rs
crates/focusa-cli/src/commands/workpoint.rs
crates/focusa-cli/src/commands/trajectory.rs
crates/focusa-cli/src/commands/focus.rs
crates/focusa-cli/src/commands/hlt.rs
crates/focusa-cli/src/commands/context_cognition.rs
crates/focusa-cli/src/commands/memory.rs
crates/focusa-cli/src/commands/turns.rs
crates/focusa-cli/src/commands/gate.rs
crates/focusa-cli/src/commands/audit.rs
crates/focusa-cli/src/commands/cleanup.rs
crates/focusa-cli/src/commands/daemon.rs
crates/focusa-cli/src/commands/uninstall.rs
crates/focusa-cli/src/commands/tui.rs
crates/focusa-cli/src/commands/constitution.rs
crates/focusa-cli/src/commands/threads.rs
crates/focusa-cli/src/commands/tokens.rs
crates/focusa-cli/src/commands/ecs.rs
crates/focusa-cli/src/commands/call_stack.rs
crates/focusa-cli/src/commands/deck.rs
crates/focusa-tui/src/main.rs
docs/current/CLI_REFERENCE_CURRENT.md
README.md
docs/RELEASE_INSTALL_POSTCARD.md
docs/GTM_FIVE_MINUTE_PROOF.md
```

---

## 30. Tests

### 30.1 Static tests

```text
tests/spec_cli_curated_help_static_test.sh
tests/spec_cli_project_dashboard_static_test.sh
tests/spec_cli_project_discovery_static_test.sh
tests/spec_cli_project_registry_static_test.sh
tests/spec_cli_project_new_static_test.sh
tests/spec_cli_project_templates_static_test.sh
tests/spec_cli_project_settings_static_test.sh
tests/spec_cli_status_operator_scoped_static_test.sh
tests/spec_cli_no_unscoped_operator_calls_static_test.sh
tests/spec_cli_first_mission_static_test.sh
tests/spec_cli_scope_safety_expansion_static_test.sh
tests/spec_cli_pairing_alias_static_test.sh
tests/spec_cli_deprecation_alias_static_test.sh
tests/spec_cli_uninstall_keep_flags_static_test.sh
tests/spec_cli_cleanup_safe_static_test.sh
tests/spec_cli_stop_output_static_test.sh
tests/spec_tui_api_route_parity_static_test.sh
tests/spec_cli_cross_phase_smoke_test.sh
```

### 30.2 Live tests

```bash
focusa project discover --json
focusa project new --working-dir "$(mktemp -d)" --name demo-project --git --yes --json
focusa project use demo-project
focusa project current --json
focusa status operator --json
focusa first-mission --project demo-project --yes --json
```

### 30.3 Unsafe-root tests

```bash
focusa project add --project-root /
focusa project add --project-root /root
focusa project add --project-root /home/wirebot
focusa project add --project-root /root/pi-mono
focusa project new --working-dir / --name bad
```

Expected:

```text
blocked with CLI_SCOPE_REJECT
```

### 30.4 Multi-project isolation

```bash
focusa project add --project-root /home/wirebot/focusa --alias focusa
focusa project add --project-root /home/wirebot/uiai-engine --alias engine
focusa project use focusa
focusa status operator --json > /tmp/focusa.json
focusa project use engine
focusa status operator --json > /tmp/engine.json
```

Expected:

```text
- project roots differ
- fingerprints differ
- workpoint/trajectory requests are scoped
- no bleed between project status packets
```

---

## 31. Acceptance criteria

The spec is accepted only when all are true:

```text
1. focusa -h shows curated Start Here UX.
2. focusa help all exposes full command inventory.
3. focusa help migration documents old → new command mapping.
4. focusa project opens interactive project dashboard.
5. focusa project discover finds likely safe projects.
6. focusa project list/use/bind/switch/current/status/remove exists.
7. focusa project new creates blank projects with Focusa root config.
8. focusa project templates list/show works.
9. focusa project settings works.
10. selected CLI project is explicitly convenience-only.
11. active-project observability exists.
12. hidden active-project authority does not exist.
13. status operator uses scoped API calls.
14. first-mission works end-to-end.
15. onboard/setup wizard uses project dashboard/discovery.
16. scope safety covers all scoped commands or marks unsupported ones advisory/global.
17. pairing commands are consolidated.
18. old commands warn but work.
19. lifecycle namespace exists.
20. setup namespace exists.
21. runtime namespace exists.
22. evidence namespace exists.
23. agent namespace exists.
24. work namespace exists.
25. constitution is reachable under focusa agent constitution and old alias works with warning.
26. threads is reachable under focusa work threads and old alias works with warning.
27. tokens is reachable under focusa runtime tokens and old alias works with warning.
28. ecs is reachable under focusa evidence ecs and old alias works with warning.
29. call-stack remains named call-stack; no new calls namespace is introduced.
30. focusa focus with no subcommand renders current focus view.
31. focusa stack warns and routes to focusa focus stack.
32. focusa deck remains the user-facing Mission Deck launcher.
33. focusa tui remains available as technical TUI command.
34. deprecated aliases have explicit 90-day sunset metadata.
35. every implementation phase runs the cross-phase smoke suite.
36. TUI/API route mismatches are fixed.
37. cleanup --safe works.
38. stop output is not contradictory.
39. uninstall keep flags are functional.
40. docs and tests are updated.
41. two projects can be selected and inspected without state bleed.
42. no CLI profile becomes daemon canonical authority without explicit scoped API verification.
```

---

## 32. Final product posture

Focusa CLI should feel complete:

```text
focusa project
  tells me where I am

focusa project new
  lets me create a new Focusa-aware project

focusa first-mission
  proves the product

focusa status operator
  gives me active-project observability

focusa deck
  gives me the mission cockpit
```

Complete rule:

```text
Focusa may remember what the operator selected.
Focusa may show what is active.
Focusa must verify before it mutates.
```
