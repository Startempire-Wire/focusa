# SPEC 135 MISSION CANVAS — FULL AGENT HANDOFF

**Target repository:** `Startempire-Wire/focusa`  
**Current implementation branch / PR:** `feature/spec-135-context-connectors-b2` / draft PR #110  
**Primary task:** Correct all Mission Canvas drift and implement the original Pi-controlled light-switch vision without creating a new competing specification authority.  
**Required outcome:** A Focusa-owned rich Mission Canvas professional GUI that opens or focuses directly from Pi, binds to the same live Pi Session and Attachment, and can be turned off without losing or restarting any canonical state.

---

## 0. Stop and reset your interpretation before writing code

Do not continue from the assumption that the current full-screen terminal component, sidebar, card dashboard, Markdown vertical projection, transcript C.R.I.S.T. stage, process-local layout map, screenshot, or handwritten proof JSON is the finished Mission Canvas GUI.

Those artifacts may contain useful terminal-projection, schema, lifecycle, and integration scaffolding. Preserve useful work, but classify it truthfully.

The required product is:

```text
Pi terminal interaction
        ⇅ one operator-controlled light switch
Focusa-owned rich Mission Canvas professional GUI
```

Canvas switching changes presentation only. It does not change, restart, clone, recreate, transfer, or infer:

```text
ProjectRootKey
WorkstreamKey
Instance
Session
Attachment
harness-native Pi session
model stream
tool runtime
transcript and tool history
Pi editor draft
Canvas draft
Trajectory
Workpoint
task/provider state
open and focused Work Surfaces
Canvas layout/groups/splits
Steering Queue
Follow-up Queue
Evidence
Receipts
authority
permissions
approvals
contention
browser session/context/target identity
durable event cursor/history
```

---

## 1. Mandatory authority preflight

Before changing Mission Canvas, Pi UI, Work Surfaces, workspace profiles, renderer code, C.R.I.S.T. generated UI, proof artifacts, closure state, or related tests, read in this order:

1. `AGENTS.md`
2. `docs/135-series-current-manifest.md`
3. `docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml`
4. `docs/agent/spec135-implementation-acceleration-directive.md`
5. the affected existing Spec 135 documents, especially:
   - `docs/135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md`
   - `docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md`
   - `docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md`
   - `docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md`
   - `docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md`
   - `docs/135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md`
6. current machine-readable closure DAG, feature ledger, parity matrix, proof matrix, acceptance report, Evidence and Receipts
7. current PR #110 body and branch state

Do **not** create `135L`, another lettered companion, or another prose document that competes with the current manifest and machine contract.

When wording conflicts:

```text
docs/135-series-current-manifest.md
+ docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml
→ govern host, renderer, toggle, layout and proof interpretation
```

---

## 2. Mandatory repository and continuity preflight

Before any durable change or after context loss:

```bash
git fetch origin
git status
```

If the remote moved while local changes exist:

```bash
git stash
git pull --rebase
git stash pop
```

Resolve conflicts before continuing.

Then:

1. verify `project_root + continuity_id` through the canonical Focusa identity tools;
2. resume the canonical Trajectory;
3. resume or create the exact Workpoint;
4. record the current worktree and writer lease;
5. record both presentation axes described below;
6. inspect the current remote branch before modifying or deleting existing implementation.

A Git worktree is a typed working subpath under ProjectIdentity. It is not a separate project authority.

Do not work from transcript memory, cached aliases, screenshots, predictions, or source comments as authority.

---

## 3. Write the work-item declaration before implementation

Every Mission Canvas UI slice must include a declaration equivalent to:

```yaml
requirement_refs:
  - SPEC135-MISSION-CANVAS-RICH-HOST
  - SPEC135-PI-CANVAS-TOGGLE
  - SPEC135-WORK-SURFACE-MULTIPLEXING

presentation_contract:
  interaction_mode: canvas-guided
  host_renderer: focusa_pi_rich_window
  surface_kind: project_overview | pi_session | uiai_browser | silent_session | document | research | provider_item | evidence | custom
  rich_gui_required: true
  terminal_fallback_required: true
  canonical_shell_regions:
    - work_surface_strip
    - focused_work_surface_with_focusa_right_inspector
    - work_rail
    - steering_queue
    - follow_up_queue
    - prompt_editor
  continuity_invariants:
    - project_root_key
    - workstream_key
    - session_id
    - attachment_id
    - harness_session_ref
    - transcript_and_tool_history
    - unsent_pi_editor_draft
    - unsent_canvas_draft
    - workpoint
    - trajectory
    - open_work_surfaces
    - canvas_layout
    - steering_queue
    - follow_up_queue
    - evidence
    - receipts
    - durable_event_cursor

reuse_assessment:
  existing_focusa_owner:
  existing_uiai_owner:
  existing_pi_owner:
  decided_framework:
  framework_version_ref:
  license:
  notice_required:
  conformance_fixture:
  integration_mode: adopt | wrap | configure | extend | custom
  custom_code_justification:

primitive_submission:
  canonical_owner:
  reusable_primitive:
  crist_specific_projection:
  core_change:
  api_change:
  generated_contract_change:
  uiai_change:
  client_change:
  migration:
  proof:

generated_ui:
  surface_kind:
  operation_ids: []
  catalog_components: []
  action_bindings: []
  durable_event_cursor:
  primary_action:
  autosave_behavior:
  resume_behavior:
  recovery_states: []
  terminal_fallback:
  accessibility_tests: []
  uiai_eval_scenarios: []
  evidence_requirements: []
  receipt_requirements: []
```

Do not implement a UI slice until this declaration is specific enough to determine ownership, state, routing, fallback behavior and proof.

---

## 4. Product contract: Canvas OFF, Canvas ON and headless

### 4.1 Canvas OFF — terminal-guided

```text
interaction_mode = terminal-guided
rich Canvas renderer = absent
stock Pi = primary interaction surface
Focusa runtime = active
```

Requirements:

- preserve normal Pi transcript, tools, editor and shortcuts;
- provide concise Focusa scope/readiness/Workpoint/Evidence/next-safe-action guidance;
- do not repeatedly invite the operator to enable Canvas;
- do not stop or recreate any runtime work;
- do not mutate profile defaults because a rich renderer is absent.

### 4.2 Canvas ON — canvas-guided in Focusa-enhanced Pi

```text
interaction_mode = canvas-guided
host_renderer = focusa_pi_rich_window
```

Requirements:

- Pi owns the command, shortcut or tool;
- Pi resolves the exact `ProjectRootKey + WorkstreamKey + Session + Attachment`;
- start or reuse the local rich host;
- bind the current Pi session as a live `pi_session` Work Surface;
- restore the Canvas presentation state and unsent Canvas draft;
- focus the rich window;
- keep the model and tool execution in the existing Pi/runtime owners;
- consume canonical Focusa API, Operation Registry, replay and live events;
- never create another Focusa runtime or session manager.

### 4.3 Canvas OFF after Canvas ON

Requirements:

- persist presentation state and the unsent Canvas draft;
- hide, unmount or close only the rich projection;
- return keyboard focus to stock Pi;
- keep background sessions and current agent execution active;
- preserve the Pi editor draft;
- do not recreate the Workpoint or reconstruct authority from chat history.

### 4.4 Headless

```text
interaction_mode = headless
host_renderer = headless_none
```

No human UI calls, windows, prompts, notifications or renderer activation are permitted.

---

## 5. Interaction mode and host renderer are independent axes

Never infer the renderer solely from `interaction_mode`.

```text
canvas-guided + Focusa-enhanced Pi
→ focusa_pi_rich_window

canvas-guided + terminal-only compatibility environment
→ pi_terminal_projection
→ visibly labeled terminal fallback

canvas-guided + UIAI Engine Cockpit
→ Focusa projection hosted in the distinct UIAI-owned product

terminal-guided
→ stock Pi; rich Canvas absent

headless
→ headless_none
```

Valid host renderer IDs:

```text
focusa_pi_rich_window
uiai_engine_cockpit
mission_deck_web
pi_terminal_projection
native_tui
menubar_peek
headless_none
```

The current full-screen box-drawing Pi component is a `pi_terminal_projection` unless and until it is replaced by a real local graphical host. Do not label it `full_gui`, `graphical_gui`, `rich_desktop_shell`, or `complete_a2ui_canvas`.

---

## 6. Naming constitution

Use these names exactly:

```text
Focusa Mission Canvas
  Complete interactive Focusa professional-workspace projection.

Work Surface
  One tab, pane, split or detached window inside Mission Canvas.

Spec Workbench
  The distinct Spec 120 adversarial specification environment.

UIAI Engine Cockpit
  The distinct UIAI-owned rich desktop product.
```

The word **Cockpit** is reserved exclusively for **UIAI Engine Cockpit**.

Do not introduce:

```text
Focusa Cockpit
Pi Cockpit
Mission Cockpit
Professional Cockpit
Cockpit Mode
Current Workspace Cockpit
```

---

## 7. Rich-host architecture

The required implementation shape is:

```text
Pi command / shortcut / tool
→ exact ProjectRootKey + WorkstreamKey + Session + Attachment binding
→ start / reuse / focus / hide / close Focusa Mission Canvas local window
→ SvelteKit 2 / Svelte 5 rich shell
→ Focusa Svelte Custom Elements
→ A2UI web_core + maintained Lit renderer for generated surfaces
→ canonical Focusa API and Operation Registry
→ durable SQLite replay + live broadcast tail + scoped invalidation
```

The rich host:

- is Focusa-owned;
- may be implemented as a Tauri webview/window or equivalent approved local webview;
- is controlled directly from Pi;
- is not another runtime;
- is not UIAI Engine Cockpit;
- does not own browser credentials or browser context storage;
- does not own model or tool execution;
- does not acquire canonical authority;
- does not make UI focus equal canonical session activity.

Before adding any new desktop, animation or UI dependency:

1. search for an existing approved primitive;
2. apply `Adopt → Wrap → Configure → Extend → Custom`;
3. document version, license and notices;
4. add a conformance fixture;
5. update the framework and SBOM/license inventories;
6. do not silently add a second runtime or framework.

---

## 8. Canonical Mission Canvas anatomy

All rich Mission Canvas hosts preserve this exact ordered composition:

```text
1. Work Surface strip
2. Focused Work Surface + Focusa right inspector
3. Work Rail
4. Steering Queue
5. Follow-up Queue
6. Prompt Editor
```

Reference:

```text
┌ WORK SURFACES ─────────────────────────────────────────────────────────────┐
│ Overview · Pi · UIAI · Silent · Documents · Research · Evidence · custom │
├────────────────────── FOCUSED WORK SURFACE ───────┬──── FOCUSA ──────────┤
│ Pi transcript, browser, document, artifact,       │ project and scope     │
│ research, generated C.R.I.S.T., comparison,       │ Session/Attachment    │
│ Evidence, terminal, code diff, chart or redline   │ Workpoint / next work │
│                                                    │ proof / authority      │
│                                                    │ contention / recovery  │
├────────────────────────────────────────────────────┴───────────────────────┤
│ WORK RAIL · surface-local / project aggregate / labeled advisory          │
├────────────────────────────────────────────────────────────────────────────┤
│ STEERING QUEUE · explicit Attachment/session recipient                    │
├────────────────────────────────────────────────────────────────────────────┤
│ FOLLOW-UP QUEUE · explicit Attachment/session recipient                   │
├────────────────────────────────────────────────────────────────────────────┤
│ PROMPT EDITOR · focused Work Surface is the default recipient              │
└────────────────────────────────────────────────────────────────────────────┘
```

Optional host/profile regions may include:

```text
global scope bar
left activity rail
detached inspector
secondary pane
compact launcher
```

These are optional. They must not:

- be presented as canonical requirements;
- replace any of the six required regions;
- reorder the six-region invariant;
- introduce another state store;
- become required merely because they appear in a visual mockup.

The active Pi transcript is one `pi_session` Work Surface. It is not the entire Canvas shell.

---

## 9. Work Surface requirements

A Work Surface is durable and rehydratable.

Supported kinds:

```text
project_overview
pi_session
uiai_browser
silent_session
document
research
provider_item
evidence
custom
```

Required operations:

```text
open
focus
pin
unpin
group
reorder
split_horizontal
split_vertical
compare
suspend_projection
rehydrate
close_projection
```

Additional governed actions may pause or terminate an underlying runtime, but **close projection** must not implicitly terminate the underlying session.

Requirements:

- every mutation target uses an explicit Attachment;
- visual focus is presentation state, not canonical authority;
- each tab/pane shows bounded project/workstream, session kind, activity, health, unread, approval, conflict, writer, isolation and proof indicators;
- layout, groups, splits, pinned state and focus persist and rehydrate;
- missing runtime references produce explicit recovery states;
- never manufacture a new session because an old projection cannot reconnect;
- use real panes and real durable layout data—not Markdown drawings or process-local maps.

---

## 10. Focusa right inspector

The inspector must project canonical state relevant to the focused Work Surface, including as applicable:

```text
mission and exact scope
ProjectRootKey / WorkstreamKey
Instance / Session / Attachment
harness session reference
current Workpoint
next safe action
current and upcoming work
authority and permissions
approval state
Evidence and Receipts
contention and proposals
writer lease and worktree
browser session/context/target and isolation posture
freshness
degraded state
recovery point and actions
```

The inspector is a projection and action surface. It does not become canonical authority.

Controls must be placed near the state they affect. Avoid a generic settings dump.

---

## 11. Work Rail and routing queues

### Work Rail modes

```text
Surface-local
  Workpoint, work items, Evidence and queues for the focused Work Surface.

Project aggregate
  Work and sessions under one verified ProjectRootKey.

Cross-project advisory
  Explicitly labeled read-only aggregation with no implicit mutation target.
```

A task is not steering. Steering is not a task. Follow-up is not an upcoming provider work item.

### Steering Queue

- delivered to an explicit Attachment/session at the next safe active-turn boundary;
- recipient, scope, role and authority remain visible;
- broadcast requires a recipient preview;
- accidental implicit broadcast is forbidden.

### Follow-up Queue

- delivered to an explicit Attachment/session after its current run;
- remains distinct from steering and tasks;
- survives toggle, restart and reconnect.

### Prompt Editor

- defaults to the focused Work Surface;
- visibly shows recipient and scope;
- supports explicit rerouting;
- preserves unsent drafts across Canvas transitions.

---

## 12. Dynamic registries and workspace profiles

Required registries include:

```text
PanelRegistry
HomeCanvasRegistry
WorkSurfaceRendererRegistry
WorkSurfaceActionRegistry
ArtifactRendererRegistry
ActionRegistry
TerminologyRegistry
ThemeRegistry
IconRegistry
HistoryProjectionRegistry
WorkspaceProfileRegistry
DomainSemanticBindingRegistry
SessionKindPresentationRegistry
```

Forbidden:

```js
if (workspace === "legal") renderEntireLegalApplication();
```

Required flow:

```text
workspace manifest
→ domain/session contracts
→ bounded read model
→ profile resolver
→ Work Surfaces and panels
→ artifact renderers
→ terminology
→ theme and icon tokens
→ resolved projection
```

Unknown panel, renderer, semantic type or domain-pack IDs must render an explicit degraded-state card and migration/recovery guidance. Do not silently omit or reinterpret them.

---

## 13. Professional vertical recomposition

Profiles include:

```text
general
software
legal
markets
research
custom
composite profiles
```

A profile switch preserves identical canonical state and must recompose:

```text
layout geometry
panel composition and order
terminology
artifact renderer bindings
Evidence and verification emphasis
history projection
iconography
density
controls
next-action emphasis
```

Color-only switching is nonconformant.

Hard-coded Markdown templates are not vertical workspaces.

Examples:

### Software

- code diff and merge views;
- tests, CI, dependencies, diagnostics and worktree emphasis;
- dense but legible geometry;
- terminology such as Repository, Task, Diff and Proof.

### Legal

- redline, authorities, matters, citations, deadlines and document hierarchy;
- source/citation and approval emphasis;
- legal terminology and evidence policy.

### Markets

- thesis, catalysts, watchlists, sources, price bands, risk and portfolio context;
- freshness and source provenance are prominent;
- no invented financial authority from visual focus.

### Research

- claims, sources, contradiction, confidence, evidence graph and synthesis;
- clear candidate/canonical distinction;
- citation and freshness emphasis.

---

## 14. C.R.I.S.T. generated UI boundary

A2UI v0.9.1, `@a2ui/web_core/v0_9`, `@a2ui/lit/v0_9`, and trusted Focusa Svelte Custom Elements render generated C.R.I.S.T. interaction surfaces **inside Work Surfaces**.

A generated C.R.I.S.T. surface may own:

```text
stage interaction surface
plain-language explanation
trusted inputs
validation presentation
generated action bindings
progress and recovery cards
```

It does not own:

```text
complete Mission Canvas shell
canonical runtime
permission authority
history
tool execution
another message processor
arbitrary HTML or JavaScript
```

The following are incomplete:

```text
transcript-only stage
Markdown/JSON dump
static form presented as generated UI
CLI selector with decorative panel
```

All generated actions route through the Operation Registry with exact scope, capability and permission projection.

---

# APPLE-STYLE DESIGN AND INTERACTION CONTRACT

The following requirements apply to the rich Mission Canvas without changing the product architecture above.

They are interaction-quality constraints, not a new product or state model.

---

## 15. Human outcomes

Design each interaction to support:

```text
Safety / predictability
Understanding
Achievement
Joy
```

The intended emotional posture is:

```text
calm
capable
direct
trustworthy
precise
```

Do not add decorative motion, glass, sound, haptics or color that competes with project truth.

---

## 16. Response and latency

The interface must acknowledge input immediately.

Requirements:

- pressed controls visibly respond on pointer-down, not only after click/release;
- continuous interactions update continuously;
- no artificial delay is introduced for visual polish;
- do not block input while waiting for a transition;
- route actions immediately into clear pending/committed/error states;
- show status for actions that outlive the press;
- validate inline rather than only on final submission;
- latency must never make an operator wonder whether steering, follow-up, focus, split or toggle was accepted.

Suggested press feedback token:

```text
pressed scale: approximately 0.97
response: approximately 100 ms
```

This is feedback only. The governed action still commits according to the correct semantic event, generally on release/activation.

---

## 17. Direct manipulation

For:

```text
pane resizing
split divider movement
tab reordering
Work Surface grouping
drawer/sheet movement
detached panel repositioning
```

Requirements:

- track the pointer 1:1;
- respect the exact point where the user grabbed the object;
- use Pointer Events and pointer capture where supported;
- track a short position/time history for release velocity;
- preserve object identity during movement;
- show the prospective destination continuously;
- never jump the object center under the pointer;
- retain keyboard-operable alternatives.

A small movement threshold—approximately 10 px—is appropriate before committing a drag direction. Do not delay simple taps unnecessarily.

---

## 18. Interruptibility

This is a hard requirement for gesture-driven motion.

- never lock out input during a transition;
- allow a moving panel, tab or sheet to be grabbed and redirected;
- start a new motion from the current on-screen presentation value;
- do not restart from a stale logical target;
- preserve and blend velocity when retargeting;
- decompose independent X and Y movement where necessary;
- Canvas ON/OFF focus handoff must remain cancellable and recoverable;
- a closing inspector or drawer may be reopened before it finishes closing.

Do not rely on fixed CSS keyframes for interactions that must be grabbed or reversed.

---

## 19. Spring behavior

Use behavior-driven springs for touchable/dragged objects.

Design tokens:

```text
Default UI movement:
  damping ratio: 1.0
  response: 0.3–0.4 seconds
  overshoot: none

Momentum-driven drawer/sheet/reposition:
  damping ratio: approximately 0.8
  response: approximately 0.3–0.4 seconds
  slight overshoot only when the gesture supplied momentum
```

Rules:

- default to critically damped motion;
- do not bounce menus, alerts or panels that did not receive a momentum gesture;
- do not confuse spring response with a fixed duration;
- retarget springs from current presentation value and velocity;
- the exact library is subordinate to the behavior contract.

Do not silently add Motion, Framer Motion or another animation dependency. First perform the required reuse assessment, conformance fixture, license review and framework-contract update.

---

## 20. Velocity handoff and momentum projection

At gesture release:

1. estimate release velocity from recent pointer history;
2. project the likely resting endpoint;
3. select the nearest valid snap point to the projected endpoint;
4. start the spring with the release velocity;
5. preserve semantic constraints and valid layout boundaries.

Reference projection model:

```js
function project(initialVelocity, decelerationRate = 0.998) {
  return (initialVelocity / 1000) *
    decelerationRate /
    (1 - decelerationRate);
}
```

Use this behavior for physical interactions such as Work Surface reordering, draggable sheets or detached panels where momentum adds clarity.

Do not use momentum for authority-sensitive actions such as destructive close, terminate, broadcast or approval commit.

---

## 21. Spatial consistency

Requirements:

- enter and exit along the same path;
- originate popovers, menus and sheets from the control that opened them;
- return closed elements toward their source;
- use symmetric reversible transitions;
- preserve stable placement for recurring controls;
- controls that look the same must behave the same;
- the Canvas toggle must remain in a predictable Pi location;
- the Focusa inspector remains spatially associated with the focused Work Surface;
- steering and follow-up controls remain next to their respective queues.

Do not move the same primary action between unrelated regions as profiles change.

---

## 22. Soft boundaries and rubber-banding

For overscroll, sheet edges, split limits and draggable panels:

- resist progressively rather than stopping abruptly;
- keep input feedback continuous;
- return to the legal boundary with a critically damped spring;
- never allow visual overshoot to imply a valid canonical state;
- never rubber-band authority, approvals or destructive confirmations.

Reference model:

```js
function rubberband(overshoot, dimension, constant = 0.55) {
  return (overshoot * dimension * constant) /
    (dimension + constant * Math.abs(overshoot));
}
```

---

## 23. Materials and depth

Use translucent material only where it clarifies hierarchy.

Appropriate candidates:

```text
optional global scope bar
floating toolbars
popover menus
drawers and sheets
detached inspector
compact launcher
non-blocking overlays
```

Rules:

- major content surfaces remain stable and highly legible;
- material weight communicates hierarchy;
- larger floating surfaces may use stronger blur and shadow than chips;
- do not stack light translucent surfaces on light translucent surfaces;
- keep colored text and controls on sufficiently solid layers;
- use slightly stronger type weight/contrast over translucent material;
- use edge fades or overlap shadows instead of excessive hard divider lines;
- a modal task uses a dimming scrim;
- a parallel non-blocking inspector does not dim the entire workspace;
- animate blur and scale together when materializing a floating material;
- provide reduced-transparency fallbacks.

Glass is not the product. Project truth remains the visual priority.

---

## 24. Multimodal feedback

Use sound or haptics only where the platform supports them and the feedback earns its place.

Apply:

```text
Causality
  feedback occurs on the event that caused it.

Harmony
  visual, sound and haptic feedback occur in sync.

Utility
  feedback is reserved for meaningful commit, snap, success, warning or error.
```

Do not add constant haptics or sound to routine navigation.

Potential meaningful events:

- Canvas ON/OFF completes;
- split snaps to a legal layout;
- steering is committed to an explicit recipient;
- destructive termination is confirmed;
- Evidence-backed closure completes;
- a serious error or contention state appears.

Always provide an equivalent visual state. Sound/haptic feedback is never the sole communication channel.

---

## 25. Reduced motion, transparency and contrast

Implement independent responses to:

```text
prefers-reduced-motion: reduce
prefers-reduced-transparency: reduce
prefers-contrast: more
```

Reduced motion:

- replace large slides, parallax and springs with short cross-fades or static state changes;
- remove elastic overshoot;
- retain useful opacity and color feedback;
- avoid full-viewport moving backgrounds and slow looping oscillation;
- do not abruptly flash between dark and light states.

Reduced transparency:

- increase material opacity;
- remove or reduce backdrop blur;
- retain clear region separation.

Higher contrast:

- use near-solid backgrounds;
- add defined contrasting boundaries;
- preserve semantic color distinction without relying on color alone.

Motion preferences must apply to generated A2UI surfaces and custom Focusa components, not only the shell.

---

## 26. Typography

Use the platform system font by default.

Requirements:

- use optical sizing where available;
- use size-specific tracking;
- large display text may use slightly negative tracking;
- small labels use slightly positive tracking when needed for legibility;
- body text remains near neutral tracking;
- large headings use tighter leading;
- body and instructional text use comfortable leading;
- build hierarchy from size, weight and leading together;
- use relative units so text-size changes scale layout;
- support zoom and platform text-size settings;
- do not truncate project, Workpoint, recipient, authority or error identity without a discoverable full value;
- code, IDs and exact scope may use a monospaced face.

Never apply one fixed letter-spacing or line-height to the entire application.

---

## 27. Purpose

Every visible element must earn its place.

Before adding a control, ask:

```text
What operator decision or action does this support?
Which canonical state does it project?
What happens if it is absent?
Why is it in this region?
```

Remove decorative status cards that duplicate the inspector, Work Rail or queue state.

Do not implement an optional left rail or global bar merely because the visual contains one. Implement optional chrome only when it improves wayfinding for the resolved host/profile.

---

## 28. Agency

Keep the operator in control.

Requirements:

- Canvas ON/OFF is explicit and reversible;
- close projection is separate from terminate session;
- destructive actions require impact preview where appropriate;
- common non-destructive actions support undo when practical;
- steering and follow-up always show recipient;
- broadcast always shows preview;
- the operator can interrupt and reverse presentation motion;
- personal layout preferences remain distinct from project-required safety and proof regions;
- do not trap the operator in Canvas mode.

Avoid confirmation dialogs for ordinary reversible actions. Use them only for genuinely destructive or authority-sensitive operations.

---

## 29. Responsibility

Act in the operator’s interest.

Requirements:

- request permissions at the moment they are needed;
- explain scope and consequences plainly;
- expose authority, Evidence and contention;
- prevent accidental cross-project or cross-context actions;
- never copy browser credentials or storage into Focusa prompts/events;
- label advisory and degraded states;
- do not visually imply verification that has not occurred;
- use previews for broadcast, termination, context reassignment and consequential actions;
- do not ship a risky feature merely to complete a visual.

---

## 30. Familiarity

Use established platform patterns and stable mappings.

- tabs behave like tabs;
- close controls close projections unless explicitly labeled otherwise;
- disclosure controls reveal details adjacent to their source;
- menus originate from the control that invoked them;
- keyboard focus order matches visual order;
- standard shortcuts are preserved where they do not conflict with Pi;
- the same icon, label and placement imply the same action across profiles;
- use specific labels such as `Evidence`, `History`, `Work Rail`, `Close view` and `Terminate session`.

Do not use vague umbrellas such as `Home` or `More` when the content can be named directly.

---

## 31. Flexibility

Support platform, context and ability differences.

Requirements:

- precise pointer/keyboard desktop workflows;
- narrow-window and terminal fallbacks;
- resizable regions;
- user-controlled density where allowed;
- personal Work Surface grouping and ordering;
- project-required safety/proof regions remain non-hideable;
- scalable typography;
- keyboard, screen reader and reduced-motion support;
- device-specific presentation state without overwriting shared project semantics;
- explicit degraded states when a required renderer or domain pack is unavailable.

---

## 32. Simplicity—not superficial minimalism

Show the common path first and advanced detail one level deeper.

Examples:

- Work Surface tabs show bounded indicators; full session identity appears in the inspector;
- Work Rail shows current work; full provider history is a drill-in;
- Steering shows recipient and summary; broadcast preview reveals the recipient matrix;
- common Canvas toggle is direct; advanced host diagnostics live in detail;
- the focused surface remains visually dominant;
- optional chrome uses progressive disclosure.

Do not hide important scope, authority, Evidence or recipient information merely to make the screen look sparse.

---

## 33. Craft

Treat spacing, timing, alignment, typography, iconography and transitions as governed implementation details.

Requirements:

- define tokens rather than scattered magic values;
- align icon optical sizes;
- keep hit targets accessible;
- audit clipping at all breakpoints;
- prevent layout jitter during live updates;
- virtualize large lists where required;
- use compositor-friendly motion;
- keep active focus visible;
- ensure dark/light and high-contrast themes remain legible;
- test pointer, keyboard and screen reader interactions;
- examine motion frame-by-frame;
- remove dead transitions and inconsistent easing;
- iterate after real use.

---

## 34. Delight

Delight is the result of clarity, agency, safety, continuity and craft.

The target feeling is that the Mission Canvas:

```text
opens instantly from Pi
already knows the current work
preserves everything
places the right evidence and action nearby
moves naturally
can be interrupted
never hides authority
gets out of the way when turned off
```

Do not add confetti, decorative bouncing, excessive glow, ornamental glass or ambient motion as a substitute for this feeling.

---

## 35. Wayfinding, grouping and labels

Every state must answer:

```text
Where am I?
Which Project / Workstream / Session / Attachment is this?
Which Work Surface is focused?
What else is active?
What is current work?
What is the next safe action?
Where can I go?
How do I return to Pi?
How do I close only the view?
```

Grouping rules:

- proximity implies relationship;
- controls sit near the object they affect;
- Work Rail modes are visibly distinct;
- Steering and Follow-up remain separate;
- inspector sections group canonical state by purpose;
- destructive controls are separated from routine controls.

Use direct, specific labels.

---

## 36. Implementation sequence

Follow this sequence. Do not keep expanding the terminal shell and plan to relabel it later.

```text
1. Compile and validate the interaction-mode + host-renderer contract.
2. Reconcile closure ledger states and reclassify current terminal work.
3. Implement durable Work Surface and Canvas presentation state.
4. Implement Focusa Pi rich-window lifecycle operations.
5. Implement the typed Pi bridge and exact Session/Attachment binding.
6. Implement the Svelte Mission Canvas shell with the six-region invariant.
7. Project the current Pi session as a live pi_session Work Surface.
8. Implement Work Surface inventory, focus, pinning, grouping and reordering.
9. Implement real horizontal/vertical splits and comparisons.
10. Implement suspend, close projection and rehydration.
11. Implement Focusa inspector, Work Rail, Steering, Follow-up and prompt routing.
12. Implement dynamic registries and workspace profile resolution.
13. Implement generated C.R.I.S.T. Work Surfaces through A2UI/Lit.
14. Implement UIAI browser Work Surfaces through typed references.
15. Implement Apple-style response, direct manipulation, interruptibility,
    spatial consistency, materials, typography and accessibility.
16. Prove Canvas OFF/ON continuity, reconnect, restart and draft preservation.
17. Run UIAI Engine Eval for visual, responsive, reconnect and accessibility proof.
18. Generate real Evidence and Receipts.
19. Regenerate all machine-readable ledgers and final acceptance.
```

Parallelize only after contracts stabilize. Use scoped worktrees, writer leases, explicit Workpoints and Attachment targets. Do not share a dirty writer workspace.

---

## 37. Current implementation audit and preservation policy

Audit every current PR #110 file against these categories:

```text
valid canonical foundation
valid rich-host implementation
valid terminal fallback
partial scaffold requiring migration
nonconformant representation
invalid proof
obsolete duplicate
```

Preserve and revalidate useful work such as:

- interaction-mode enum and precedence;
- canonical runtime continuity intent;
- headless no-UI behavior;
- Work Surface schema foundations;
- Session/Attachment identity;
- browser-isolation contracts;
- terminal projection components;
- API/Operation Registry foundations;
- durable stream foundations;
- dynamic registry foundations that actually conform.

Reclassify:

- `MissionCanvasShell` / `ctx.ui.custom(...)` full-screen box-drawing UI as terminal projection;
- Markdown split representations as non-rich fallback;
- process-local layout maps as non-durable scaffolds;
- transcript-only C.R.I.S.T. stages as terminal fallback;
- status-card dashboards as partial compact projections.

Remove or supersede only after verifying that no canonical behavior is lost.

---

## 38. Proof contract

### Unit and contract proof

Use for:

```text
schema
reducers
scope and identity
operation bindings
host capability resolution
layout persistence
profile resolution
permission routing
motion token calculation
reduced-motion behavior
```

### Component proof

Use for:

```text
six shell regions
Work Surface tabs
real pane mechanics
inspector sections
Work Rail modes
queue recipient routing
prompt draft preservation
profile recomposition
keyboard and screen reader behavior
pointer-down feedback
interruptible motion
reduced transparency and contrast
trusted generated components
```

### Runtime integration proof

Must exercise:

```text
Pi command
→ rich host lifecycle
→ exact same Session and Attachment
→ live transcript/tool event continuity
→ unsent draft preservation
→ Work Surface operations
→ Canvas OFF
→ same Pi session and Workpoint
```

### UIAI Engine Eval

Required for:

```text
rich GUI rendering
visual comparison
responsive breakpoints
browser-facing accessibility
reconnect behavior
generated C.R.I.S.T. interaction
browser Work Surfaces
screenshots and diagnostics
```

Focusa must not add Playwright as a parallel browser proof stack.

### Invalid rich-GUI proof

The following do not prove the rich GUI:

```text
source substring checks
handwritten pass JSON
static screenshot without runtime trace
Markdown drawing of a split
process-local layout map
box-drawing terminal shell
transcript-only C.R.I.S.T. stage
file existence alone
```

A screenshot may accompany runtime proof but cannot replace it.

---

## 39. Required continuity tests

At minimum, prove:

1. start a real Pi session;
2. enter an unsent Pi editor draft;
3. open Canvas from Pi;
4. verify same Session and Attachment;
5. verify live transcript and tool updates;
6. enter an unsent Canvas draft;
7. create, focus, pin, group and split Work Surfaces;
8. add steering and follow-up items with explicit recipients;
9. switch vertical profiles and verify actual recomposition over identical canonical state;
10. interact with a generated C.R.I.S.T. Work Surface;
11. interrupt a moving panel/split and retarget it without jumping;
12. enable reduced motion and verify cross-fade/static equivalents;
13. enable reduced transparency/high contrast and verify legibility;
14. close Canvas;
15. verify Pi draft, Canvas draft, Workpoint, queues, Evidence and layout remain;
16. reopen Canvas and rehydrate exact state;
17. reconnect/restart the client and replay from the durable cursor;
18. verify no new canonical Session or Attachment was manufactured.

---

## 40. Acceptance states

Use truthful states:

```text
implemented
partially_implemented
terminal_fallback_only
rich_host_missing
proof_missing
blocked
verified
```

Do not collapse partial or fallback states into `passed`.

Spec 135 is not complete or merge-ready until the required rich host and runtime proof exist and all affected machine-readable ledgers are regenerated.

---

## 41. Forbidden actions and claims

Do not:

- create 135L or another competing companion;
- commit directly to `main`;
- call Focusa/Pi UI a Cockpit;
- call the terminal projection the full GUI;
- hard-code separate vertical applications;
- present color-only switching as vertical recomposition;
- use visual focus as canonical authority;
- fork the Pi session manager or model/tool execution;
- create a second Focusa runtime or event history;
- make A2UI own the complete shell;
- add arbitrary generated HTML/JavaScript;
- add Playwright to Focusa;
- rewrite provider JSONL manually;
- close projections by silently terminating sessions;
- hide exact recipients for steering/follow-up;
- use bouncy motion on non-momentum actions;
- block input during transitions;
- ignore reduced-motion/transparency/contrast settings;
- declare closure from screenshots, source strings or handwritten JSON.

---

## 42. Progress report format

At the end of each meaningful slice, report:

```markdown
## Workpoint
- ID:
- ProjectRootKey:
- WorkstreamKey:
- worktree:
- writer lease:

## Presentation contract
- interaction_mode:
- host_renderer:
- surface_kind:
- rich_gui_required:
- terminal_fallback:

## Authority and scope
- Session:
- Attachment:
- exact files/lines:
- permissions:

## Implemented
- canonical runtime:
- rich host:
- terminal fallback:
- generated UI:
- profile/vertical:
- Apple interaction principles:

## Reused
- existing primitive:
- framework/version:
- integration mode:
- license/notice:

## Proof
- unit/contract:
- component:
- runtime integration:
- UIAI Engine Eval:
- Evidence:
- Receipts:

## Continuity
- same Session/Attachment:
- Pi draft preserved:
- Canvas draft preserved:
- Workpoint preserved:
- layout/queues preserved:
- reconnect/replay:

## Truthful status
- implemented | partially_implemented | terminal_fallback_only |
  rich_host_missing | proof_missing | blocked | verified

## Remaining blockers
-
```

Do not claim completion using qualitative phrases without the proof references.

---

## 43. Session completion and push discipline

Before ending a session:

1. create issues for remaining work;
2. run relevant tests, linters, contract generation and builds;
3. update Beads/issue status;
4. fetch remote and rebase safely;
5. sync Beads;
6. commit with a meaningful Conventional Commit subject;
7. push the feature branch;
8. verify the branch is up to date with origin;
9. clear stale stashes and prune where appropriate;
10. provide the progress report above.

Required final commands:

```bash
git pull --rebase
bd sync
git push
git status
```

`git status` must show the branch is up to date with origin.

Do not say “ready to push.” The agent performing the work must push it.

Build and deploy only through the canonical GitHub release pipeline defined in `AGENTS.md` and `docs/canonical-live-release-pipeline.md`. Do not deploy locally built release binaries.

---

## 44. First action now

Perform the following immediately:

```text
1. Fetch and rebase the current PR #110 branch safely.
2. Resume the canonical Spec 135 Workpoint and writer lease.
3. Read the authority files in the required order.
4. Audit the current Mission Canvas implementation and proof artifacts.
5. Produce a file-by-file reconciliation table.
6. Reclassify terminal UI and invalid proof without deleting useful foundations.
7. Create the implementation Workpoint declarations for:
   a. host-renderer resolver,
   b. Pi rich-window lifecycle bridge,
   c. rich six-region shell,
   d. live pi_session Work Surface,
   e. durable split/group/rehydration,
   f. profile registries and vertical recomposition,
   g. generated C.R.I.S.T. Work Surfaces,
   h. continuity and Apple-style interaction proof.
8. Begin with the first missing production-shaped dependency in the sequence.
9. Keep PR #110 in draft until all reopened gates are runtime-proven.
```

The success condition is not a prettier terminal interface. It is a truthful, durable and proven Pi-controlled rich Mission Canvas that feels immediate, predictable, interruptible, accessible and calm while preserving the exact Focusa authority model.
