# Focusa Workforce Widget Library and Organization Wall Specification

**Status:** Proposed implementation specification  
**Owner:** Focusa Workforce  
**Scope:** Chrome start page, sidepanel, daemon widget APIs, organization display walls, and future embeddable clients  
**Depends on:** Specs 98, 103, 109, 133, 135B, 140, 141, 149, 155, 164, 170, 173–178  
**Primary outcome:** Turn Focusa primitives into a governed, composable visual system for personal workspaces and organization-wide screens.

## 1. Product thesis

Focusa should not ship one fixed dashboard. It should expose a library of small, trustworthy visual instruments—widgets—each backed by a registered Focusa primitive and a typed daemon projection. Operators compose those instruments into views for:

1. a personal Chrome start page;
2. an in-context Chrome sidepanel;
3. a shared office/operations wall on a large monitor;
4. future web, menubar, tablet, and embedded clients.

The browser is a rendering and interaction client. The daemon remains authoritative for identity, scope, queries, permissions, scheduling, state, events, completion, and receipts. A widget may request and render data, but it may not invent roster state, infer completion, schedule work, or grant authority.

## 2. Goals

- Make the first screen explain what matters now within five seconds.
- Let operators turn widgets on/off, resize them, reorder them, and save multiple layouts.
- Provide personal, team, project, and organization views without duplicating authority.
- Support a read-only wall mode safe for large shared displays.
- Keep every visible claim source-linked, timestamped, scope-bound, and freshness-aware.
- Make unsupported, stale, unauthorized, or degraded data visibly distinct.
- Compose widgets from registered Focusa capabilities and projections only.
- Preserve graceful operation during daemon restarts, network loss, browser sleep, and partial provider failure.
- Make adding a new widget a typed contract plus registry entry, not a bespoke dashboard rewrite.

## 3. Non-goals

- Browser-owned canonical state, scheduling, permissions, or completion.
- Arbitrary client JavaScript queries against SQLite or daemon internals.
- A generic BI tool that imports uncontrolled external data into Focusa.
- Silent mutation from a display widget.
- Treating a wall display as an operator identity or approval authority.
- Allowing a layout author to expose data outside their granted organization, project, or workstream scope.
- Replacing Worksets, Workstreams, Task Plans, CallGraphs, Silent Sessions, or Work Loop authority.

## 4. User experiences

### 4.1 Personal start page

The start page is the default new-tab workspace. It provides a calm overview and quick navigation into deeper work.

Required regions:

- **Focus:** active mission, current objective, next meaningful action, and freshness.
- **Workforce:** active agents, lifecycle, health, current activity, and exact-target controls.
- **Controls:** open sidepanel, orient current tab, pause/resume permitted work, review receipts.
- **Activity:** bounded recent events with source and time.
- **Brief:** scope, blockers, approvals, and evidence gaps.
- **Customize:** widget visibility, order, size, density, theme, and layout selection.

The start page must remain useful when no daemon is connected: show a clear disconnected state, local layout controls, and safe connection actions without fabricating project data.

### 4.2 In-context sidepanel

The sidepanel is optimized for working beside a webpage:

- observe the current tab;
- orient a mission;
- create or control an exact agent target;
- inspect live events and approvals;
- open the current start-page layout or a focused widget view.

The sidepanel and start page use the same widget contracts but different default layouts and density.

### 4.3 Organization wall

Wall mode is a read-only, resilient display for a large monitor, TV, kiosk, or browser tab shared by a team.

Required characteristics:

- no mutation controls by default;
- explicit organization/project/workstream scope;
- large typography and high contrast;
- automatic rotation between approved views;
- configurable refresh cadence with server freshness timestamps;
- visible `LIVE`, `STALE`, `DEGRADED`, `OFFLINE`, and `NOT AUTHORIZED` states;
- no secrets, raw prompts, private message content, provider tokens, or sensitive output;
- browser reconnect and daemon restart recovery;
- optional kiosk URL containing an opaque, revocable wall-view reference—not a bearer token in the URL.

Wall mode may show organizational aggregates, but aggregation must be performed by the daemon and must enforce minimum-group/privacy rules before projection.

## 5. Widget model

Every widget is a versioned registry item:

```json
{
  "schema": "focusa.widget_descriptor.v1",
  "widget_id": "focusa.workforce.roster",
  "revision": 1,
  "title": "Workforce",
  "description": "Bounded agent lifecycle and health overview",
  "family": "workforce",
  "primitive_refs": ["focusa_silent_sessions", "focusa_work_loop_status"],
  "query": {
    "operation_id": "focusa.widget.workforce.roster.read",
    "request_schema_ref": "focusa.widget_query.v1",
    "response_schema_ref": "focusa.widget_projection.v1"
  },
  "allowed_surfaces": ["startpage", "sidepanel", "wall"],
  "default_size": "wide",
  "supported_sizes": ["compact", "wide", "large"],
  "refresh_policy": {"mode": "event_plus_interval", "min_interval_ms": 5000},
  "privacy_class": "project_scoped",
  "mutation": "none",
  "freshness_sla_ms": 30000,
  "fallback": "stale_snapshot_with_banner"
}
```

### 5.1 Widget rules

- `widget_id + revision` is stable and addressable.
- Every widget references one or more registered primitives.
- Every widget declares its required scopes, privacy class, supported surfaces, refresh policy, and mutation profile.
- Wall widgets must declare `mutation: none`.
- Widget output is a bounded projection, never an unbounded event or database dump.
- Unknown widget revisions fail closed and render an upgrade card.
- A widget cannot request a capability absent from its descriptor.
- A widget cannot use a client-supplied capability or permission assertion.
- A widget must render a truthful empty state when its query returns no records.
- Every projection includes `source_revision`, `generated_at`, `fresh_until`, `scope`, and `evidence_refs` where applicable.

### 5.2 Initial widget library

Implement the first library in these families:

**Orientation and focus**

- Active mission
- Current objective and next action
- Blockers and drift
- Scope/identity health
- Evidence coverage

**Workforce**

- Agent roster
- Agent health and lifecycle
- Current agent activity
- Work Loop status
- Exact-target control card for sidepanel only

**Execution**

- Workstream progress
- Workset settlement
- Task Plan frontier
- CallGraph run status
- Fanout lane status
- Queue and lease health

**Evidence and governance**

- Recent receipts
- Approval inbox summary
- Authorization decisions
- Evidence freshness
- Capability availability
- Entitlement status

**Organization wall**

- Portfolio health
- Project status matrix
- Delivery flow
- Blocker heatmap
- Workforce capacity
- Incident/degraded-services banner
- Recent milestones

The first implementation must not claim a widget is available until its underlying primitive and route are registered, assignable, scoped, tested, and live-queryable.

## 6. Query and projection contracts

Add a daemon-owned widget projection surface:

```text
GET /v1/widgets/catalog
GET /v1/widgets/layouts
POST /v1/widgets/layouts/resolve
GET /v1/widgets/projections/{widget_id}
GET /v1/widgets/stream
GET /v1/wall-views/{wall_view_id}
GET /v1/wall-views/{wall_view_id}/stream
```

The exact generated operation registry is authoritative; route names above are the design target and must be reconciled against the live route inventory before implementation.

### 6.1 Query envelope

```json
{
  "schema": "focusa.widget_query.v1",
  "request_id": "request:...",
  "widget_id": "focusa.workforce.roster",
  "widget_revision": 1,
  "scope": {
    "organization_id": "org:...",
    "project_root": "/safe/project",
    "continuity_id": "workstream:...",
    "wall_view_id": null
  },
  "as_of": "2026-08-24T17:00:00Z",
  "limit": 50,
  "cursor": null
}
```

### 6.2 Projection envelope

```json
{
  "schema": "focusa.widget_projection.v1",
  "status": "fresh|stale|degraded|offline|unauthorized|unsupported",
  "widget_id": "focusa.workforce.roster",
  "widget_revision": 1,
  "scope": {"organization_id": "org:..."},
  "generated_at": "...",
  "fresh_until": "...",
  "source_revision": "ledger:...",
  "items": [],
  "summary": {},
  "evidence_refs": [],
  "recovery": {"action": "reconnect|request_access|open_sidepanel|upgrade"},
  "next_cursor": null
}
```

A projection status is not interchangeable with a data value. `offline` must never render as zero agents, zero blockers, or completed work.

## 7. Live updates

Use one governed stream for layout-relevant invalidations and bounded projection refresh:

```text
widget_invalidated(widget_id, scope, source_revision, reason)
widget_projection_updated(widget_id, projection_revision)
wall_view_changed(wall_view_id, layout_revision)
```

Rules:

- reconnect with an opaque cursor;
- deduplicate by event ID and projection revision;
- refresh only affected widgets;
- apply exponential backoff with a bounded ceiling;
- show stale age while disconnected;
- never infer a state transition from a missing event;
- request a fresh projection after reconnect;
- wall mode must continue rendering the last verified snapshot with a stale banner.

## 8. Layouts and widget composition

A layout is a durable, versioned composition owned by the daemon:

```json
{
  "schema": "focusa.widget_layout.v1",
  "layout_id": "layout:...",
  "name": "Executive wall",
  "surface": "wall",
  "scope": {"organization_id": "org:..."},
  "revision": 3,
  "approved": true,
  "grid": {"columns": 12, "row_height": 72, "gap": 16},
  "widgets": [
    {"widget_id": "focusa.portfolio.health", "x": 0, "y": 0, "w": 8, "h": 3, "config": {}},
    {"widget_id": "focusa.workforce.capacity", "x": 8, "y": 0, "w": 4, "h": 3, "config": {}}
  ],
  "theme_ref": "theme:focusa-night",
  "rotation": null
}
```

Layout editor requirements:

- add/remove widgets from the authorized catalog;
- drag/reorder and resize within a bounded grid;
- preview fresh/stale/degraded states;
- validate scope and capability requirements before save;
- autosave only local drafts;
- require explicit confirmation for durable publish;
- keep revision history and rollback;
- support personal layouts, team layouts, and approved wall layouts;
- never allow a wall layout to include mutation widgets.

## 9. Widget library and SDK

Create a small framework-neutral widget SDK used by Chrome first and future clients later:

- descriptor validation;
- query envelope construction;
- projection status handling;
- freshness and stale-age formatting;
- safe redaction helpers;
- event invalidation subscription;
- layout validation;
- accessible loading, empty, stale, degraded, and unauthorized states;
- theme tokens and density tokens.

A widget renderer receives a projection and a render context. It does not receive raw credentials, arbitrary daemon state, or unrestricted fetch access.

## 10. Security and privacy

- Organization and project scope are daemon-resolved.
- Wall access uses revocable wall-view references with expiration and rotation.
- Wall views are read-only and cannot issue approvals or mutations.
- Aggregates enforce minimum cohort size where individual exposure would be sensitive.
- Private projects and sensitive evidence require explicit wall authorization.
- Widget configuration is validated against the server-side descriptor.
- The browser may cache bounded projections, never secrets or bearer tokens in URLs.
- Every layout publish, wall-view create/revoke, and sensitive projection access is audited.
- `can(principal, capability, context)` governs widget queries and layout mutations.

## 11. Visual and interaction design

Use a coherent Focusa visual system:

- dark-first navy canvas with luminous blue/lilac accents;
- light theme with equivalent contrast;
- large typographic hierarchy for wall mode;
- compact density for sidepanel;
- calm motion only when it communicates state change;
- no flashing or color-only status;
- status always includes text and an icon/shape;
- keyboard navigation and visible focus rings;
- reduced-motion support;
- responsive breakpoints for 320px sidepanel through 4K wall;
- widget chrome remains minimal; data receives visual priority.

## 12. Implementation plan

### Slice WIDGET-1 — Contracts and registry

Files: core widget descriptor/projection types, generated contract inputs, route/operation registry, validation tests.

Done when: descriptors validate, assignability is grounded, unknown revisions fail closed, and generated cross-surface projections agree.

### Slice WIDGET-2 — Daemon projection API

Files: API widget routes, scoped query helpers, projection cache/read model, event invalidation, audit integration.

Done when: five initial widgets return bounded truthful projections with fresh/stale/degraded/unauthorized states and evidence/source metadata.

### Slice WIDGET-3 — Chrome widget runtime

Files: extension widget SDK, layout state, projection client, stream reconnect, render-state components.

Done when: widgets render from daemon projections and never reconstruct canonical state locally.

### Slice WIDGET-4 — Start-page composer

Files: startpage UI, widget library drawer, grid layout, local draft persistence, durable layout save/publish.

Done when: operator can toggle, reorder, resize, preview, save, switch, and restore layouts with keyboard support.

### Slice WIDGET-5 — Sidepanel integration

Files: sidepanel widget slots and navigation links.

Done when: start-page layouts can open focused sidepanel views and sidepanel actions return to the exact scoped layout.

### Slice WIDGET-6 — Wall views

Files: wall-view API, read-only wall renderer, kiosk bootstrap, rotation, high-density theme, reconnect behavior.

Done when: a revocable wall view can run unattended for a full test window, survives daemon restart/network interruption, and never exposes secrets or mutation controls.

### Slice WIDGET-7 — Organization administration

Files: organization layout ownership, role-based access, approval/publish flow, audit and revocation UI.

Done when: an organization can create, approve, publish, revoke, and rollback wall layouts with scope-safe access.

## 13. Testing and proof requirements

Every widget and surface requires:

1. versioned contract;
2. producer unit/property tests;
3. consumer rendering tests;
4. cross-version interop tests;
5. live end-to-end proof.

Required test groups:

- descriptor grounding and dead-capability exclusion;
- projection envelopes and status truthfulness;
- scope mismatch and authorization denial;
- layout validation and revision idempotency;
- widget toggle/reorder/resize persistence;
- keyboard and reduced-motion accessibility checks;
- SSE cursor replay, reconnect, dedupe, and stale fallback;
- daemon restart and browser sleep recovery;
- wall-view revocation and expired-reference denial;
- privacy aggregation and secret redaction;
- two-daemon federation with source identity preserved;
- Chrome MV3 deterministic build and package inspection;
- 1080p, 1440p, 4K, narrow sidepanel, and light/dark visual snapshots.

Acceptance evidence must include handles for catalog, layout revision, projection, stream, authorization decision, wall-view lifecycle, and final E2E receipt.

## 14. Rollout

1. Ship widget contracts and five read-only projections behind a development feature flag.
2. Ship start-page composer with local drafts only.
3. Add durable personal layouts after revision/idempotency proof.
4. Add approved wall views for one organization scope.
5. Add rotation, federation, and organization administration after privacy and revocation proof.
6. Promote widgets individually only when their producer, consumer, interop, and live E2E proofs are green.

No release may claim “organization wall” support while wall authorization, revocation, stale behavior, and daemon restart recovery remain unproven.

## 15. Risks and mitigations

- **Dashboard becomes a second authority:** daemon-owned projections and no client scheduling/state reconstruction.
- **Widget sprawl:** registry review, primitive references, owner, revision, and acceptance proof required.
- **Wall leaks private information:** explicit wall scope, aggregate rules, read-only policy, redaction, revocation.
- **Stale screen creates false confidence:** age banner, source revision, status text, and fail-closed empty states.
- **Layout customization becomes destructive:** local drafts, explicit publish, approval, revision history, rollback.
- **Large-screen overload:** curated defaults, density themes, rotation, bounded item counts, and visual hierarchy.
- **Multi-daemon confusion:** source daemon identity displayed and preserved in every projection.

## 16. Definition of amazing

A new operator opens a tab and immediately understands what Focusa is doing, why it matters, what is blocked, and what they can safely do next. A team can compose a shared wall that reflects real project and workforce health without exposing private content. Every number, color, and status is explainable, scoped, fresh, and traceable to daemon authority. The same governed widget can move from a personal start page to a sidepanel to a 4K organization wall without changing its truth model.
