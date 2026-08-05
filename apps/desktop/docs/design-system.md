# Focusa Desktop Design System

Status: active implementation standard

Adopted from: `/Volumes/Macintosh HD/Users/vsmith/focusa-desktop-design-handoff.md`

Target: `apps/desktop/` (the handoff's `apps/menubar/` candidate is intentionally not used)

## Product character

Focusa Desktop is a calm cognitive cockpit: dark, precise, restrained, information-dense, and truthful. Strong contrast and luminous treatment are reserved for current focus, actionable readiness, warnings, failures, and short-lived completion feedback. Visual selection never creates Scope, Workstream, Continuity, Attachment, Session, or runtime authority.

## Architecture

### Tokens

`static/styles/tokens.css` is the source for:

- dark neutral and semantic color roles;
- accent, cyan, violet, success, warning, and error tones;
- 4px-based spacing and canonical section/cluster/control gaps;
- nested control/card/panel radii;
- card and popover elevation;
- bounded SF system typography;
- micro, fast, normal, and slow motion timings.

Components must consume semantic tokens rather than introduce one-off palette, spacing, radius, or transition values. New tokens require a reusable semantic role.

### Shared primitives

| Primitive | Contract |
| --- | --- |
| `Icon.svelte` | One typed outline family, currentColor, 14/16/18/20 sizes |
| `IconButton.svelte` | 30px hit target, tooltip, label, pressed and disabled states |
| `Surface.svelte` | Neutral, raised, and rare spotlight variants |
| `SectionHeader.svelte` | Eyebrow, title, bounded description, optional action |
| `Stack.svelte` | Canonical section, cluster, and control gaps |
| `StatusBadge.svelte` | Neutral, ready, watch, blocked, and error semantics |
| `StatePanel.svelte` | Loading, empty, ready, stale, blocked, and error surfaces |
| `ThinkingOrb.svelte` | DPR-aware, visibility-aware semantic ambient state with static reduced fallback |

`BorderBeam` is intentionally deferred until an active semantic surface requires it. The UIAI reference implementation is uncommitted and not pixel-accepted; importing its generated implementation now would add more than 2,000 lines without a justified use. Any later port must include its MIT license, generated styles, shared pulse driver, offscreen pause behavior, and native WebView proof.

### Motion

`src/lib/ui/motion.ts` provides the content-pane `scene` transition and the `system | full | reduced` preference contract.

Rules:

- keep titlebar, interface switch, and navigation geometry stable;
- transition selected content only;
- use opacity and transform, never animated width, height, margin, or padding;
- direct presses use a restrained `.98` scale;
- routine polling does not remount or replay page transitions;
- system mode obeys the operating system;
- reduced mode uses short crossfades and static ambient rendering;
- full mode enables normal direct and ambient motion.

## Sidebar contract

The Desktop sidebar adopts the interaction quality—not the information architecture—of the UIAI Engine Cockpit:

- persistent expanded and compact modes;
- 208–320px pointer resize in expanded mode;
- 64px responsive icon rail;
- grouped disclosure and local presentation preferences;
- keyboard toggle with `[` outside text-entry controls;
- one typed icon family and accessible tooltips;
- visible hover, active, pressed, and focus states;
- no sidebar in Agent TUI mode.

Sidebar state is a local presentation preference only. It cannot grant project or Workstream authority.

## Truthful state presentation

Every asynchronous or unavailable surface must select one explicit state: `loading`, `empty`, `ready`, `stale`, `blocked`, or `error`. Styling can clarify a verified state but cannot promote advisory, degraded, stale, or blocked data.

Thinking Orb animation reflects an already-determined state. It never infers daemon, Workpoint, or execution status. Glow is prohibited on ordinary metadata and neutral cards.

## Typography and spacing

- Display and heading tracking: `-0.02em` to `-0.012em`.
- Body copy remains near zero tracking and at most 68 characters wide where practical.
- Eyebrows use uppercase only for short categories at `0.06–0.08em` tracking.
- Durations, counts, timestamps, latency, and resource metrics use tabular numerals.
- Peer surfaces use the canonical 16px section gap.
- Related content uses 8–10px cluster spacing.
- Nested radii step down from panel 16px to card 12px to control 8px.
- Negative-margin overlap and touching peer cards are prohibited.

## Accessibility

- All icon-only controls have accessible labels and visible tooltips.
- Focus rings use the bright accent token and remain visible in dark mode.
- Disabled controls remain distinguishable from unavailable state text.
- Motion preference never hides state or removes required feedback.
- Keyboard navigation and reduced-motion behavior are acceptance requirements.

## Verification

Visual acceptance is exclusively performed through UIAI Engine against `http://127.0.0.1:1430/`, followed by one bounded native Tauri verification when accumulated native changes justify it. Compilation and string-presence tests are necessary but do not constitute visual proof.

Required evidence for a migrated surface:

1. normal-motion wide screenshot;
2. compact/responsive screenshot;
3. reduced-motion interaction check;
4. keyboard and focus check;
5. zero console errors, exceptions, and failed requests;
6. native WebView evidence for canvas effects such as Thinking Orb or BorderBeam.

## Adoption status

Implemented in the first design-system slice:

- semantic tokens and canonical spacing/radius/type/motion scales;
- typed Icon, IconButton, Surface, SectionHeader, Stack, StatusBadge, StatePanel, and ThinkingOrb primitives;
- stable-shell keyed content transition;
- system/full/reduced motion contract;
- typed sidebar icons replacing provisional Unicode controls;
- truthful blocked workspace StatePanel;
- semantic daemon Thinking Orb;
- removal of global transition/animation suppression;
- sidebar local-preference boundary.

Incremental migration still required:

- move remaining legacy `styles.css` literals onto semantic tokens;
- migrate Mission Canvas cards and headers to shared primitives;
- replace remaining arrows and decorative glyphs in Mission Canvas identity content;
- add an operator-facing motion preference control;
- run UIAI normal, compact, reduced-motion, and keyboard acceptance;
- run native Thinking Orb acceptance in the next bounded native build;
- consider BorderBeam only when a verified active Workpoint surface exists.
