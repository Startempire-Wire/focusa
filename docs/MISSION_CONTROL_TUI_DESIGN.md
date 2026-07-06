# Focusa Mission Control TUI — Design

Purpose: redesign the TUI away from a flat tab collection toward a single **Mission Control** canvas with modal overlays and rich ratatui widgets. This is a real mission operations dashboard, not a tabbed terminal.

## Architecture

### Layout (always visible canvas)

```
┌─ FOCUSA · MISSION CONTROL ─────────────────────────────────────┐
│                                                                │
│  ┌─ MISSION CARD ─────────┐  ┌─ PROOF METER ───┐  ┌─ SCOPE ──┐│
│  │ Intent                 │  │ Verified        │  │Canonical ││
│  │ Keep the mission,      │  │ [#####]         │  │          ││
│  │ prove the handoff.     │  │ 5 evidence refs │  │ safe to  ││
│  │                        │  │                 │  │ act      ││
│  │ Current action:        │  └─────────────────┘  └──────────┘│
│  │ focusa workpoint resume│                                     │
│  │                        │  ┌─ MISSION LADDER ───────────────┐  │
│  │ Next safe action:      │  │ HLT: ship Focusa MVP           │  │
│  │ resume mission         │  │  └─ MLG: install + first run  │  │
│  │                        │  │     └─ STG: create walkthru   │  │
│  │ Why:                   │  │        └─ Workpoint: current  │  │
│  │ Project, work, proof,  │  │             └─ Evidence: 5    │  │
│  │ authority are visible. │  └────────────────────────────────┘  │
│  └────────────────────────┘                                     │
│                              ┌─ LEARN (3 walkthroughs) ─────┐    │
│  ⌘ n=next  /=recall          │ 1. First Mission               │    │
│  ⌘ l=learn  ?=help          │ 2. Agent Handoff               │    │
│  ⌘ a=about  q=quit          │ 3. No Proof, No Done           │    │
│                              └────────────────────────────────┘    │
│ focusa · 5fa3 · safe · ok                                     │
└────────────────────────────────────────────────────────────────┘
```

### Modal overlays (replace canvas, not stack)

- `n` → next safe action banner (transient)
- `/` → Recall modal (sources, card fields, allowed_use)
- `l` → Learn modal (walkthrough picker + detail)
- `?` → Help modal (concept overlay)
- `a` → About modal (version, build, telemetry, credits)
- `:` → Command palette (command → action dispatch)
- `Esc` or `q` → close modal / quit

### Why this is the right architecture

- **Always-visible canvas** matches the user mental model: "I'm at the mission controls."
- **Modal overlays** match how a real operator works: dive into Recall, Learn, or Help without losing the deck.
- **Rich widgets** (Sparkline, BarChart, LineGauge) make the Mission Card and Proof Meter feel alive, not just static text.
- **No top-level tab bar** means no need to scroll a horizontal tab list when many features ship.
- **Card metaphor** matches the Spec 117 evidence/walkthrough model.

### Mobile / small-screen friendliness

- **Responsive constraints only**: `[Constraint::Percentage(..), Constraint::Ratio(..), Constraint::Min(..)]` — never use `Constraint::Length(char_count)` for primary layout.
- **Card reflow**: every paragraph uses `Wrap { trim: true }` so cards never overflow on narrow terminals.
- **Sparkline + LineGauge** are scale-free (no fixed pixel width).
- **Single column fallback**: on viewports narrower than ~80 cols (a common mobile/Termux screen width), the layout collapses to a single stacked column instead of grid.
- **Hotkey compact**: key hints show as compact `⌘ n next` pairs that fit on small footers.
- **Asciii-safe borders** (single-line); no Unicode box-drawing that breaks on legacy terminals.

### Single-column fallback (mobile)

When `area.width < 80`, the layout collapses:

```
┌─ FOCUSA · MISSION CONTROL ─────┐
│ MISSION CARD                   │
│ Intent: ...                    │
│ Current: focusa ...            │
│ Next safe action: ...          │
│ Why: ...                       │
├───────────────────────────────┤
│ PROOF METER                    │
│ Verified [#####] 5 refs        │
├───────────────────────────────┤
│ SCOPE                          │
│ Canonical · safe to act         │
├───────────────────────────────┤
│ LADDER                         │
│ HLT > MLG > STG > WP > Proof    │
├───────────────────────────────┤
│ LEARN                          │
│ 1. First Mission               │
│ 2. Agent Handoff               │
│ 3. No Proof, No Done           │
├───────────────────────────────┤
│ ⌘n /= l ? a q · focusa 5fa3    │
└───────────────────────────────┘
```

### What changes vs the old TUI

| Old | New |
|---|---|
| Flat Tab enum with 20+ variants | Single Tab::DeckHome, modal layer for everything else |
| Tab strip header | Compact header with project/continuity |
| `Tab::Recall` body | `Recall` modal overlay (Esc to close) |
| `Tab::About` body | `About` modal overlay |
| `Tab::Walkthroughs` body | `Learn` modal overlay |
| Static text proof meter | Sparkline + LineGauge progress |
| No command palette | `:` opens command palette |
| Static lines | Real updates: session elapsed, proof count, continuity hash |

### Rich widgets used

- `ratatui::widgets::Sparkline` for proof trend
- `ratatui::widgets::LineGauge` for net savings / next-action completion
- `ratatui::widgets::BarChart` for evidence type distribution
- `ratatui::widgets::Canvas` for custom FOCUSA logo/badge drawing
- `ratatui::widgets::List` + `ListState` for walkthrough picker and command palette
- `ratatui::widgets::Tabs` (the widget) for sub-tabs inside modal (NOT a top-level tab bar)
- `ratatui::widgets::Gauge` for progress fills

### State model

```rust
pub struct App {
    pub mode: Mode,              // Deck | Modal(ModalKind) | Palette
    pub modal: Option<ModalKind>,// Recall | Learn | Help | About | None
    pub modal_selection: usize,  // index for list-like modals
    pub mission: MissionCard,    // structured mission state
    pub workpoint: WorkpointState,
    pub evidence: EvidenceState,
    pub ladder: LadderState,
    pub proof_history: Vec<u8>,  // for sparkline
    pub recall_sources: Vec<String>,
    pub walkthroughs: Vec<WalkthroughSummary>,
    pub show_intro: bool,
    pub show_help: bool,
}

pub enum Mode { Deck, Modal(ModalKind), Palette }
pub enum ModalKind { Recall, Learn, Help, About }
```

### Hotkeys (replaces flat tab strip)

- `n` → next safe action banner
- `/` → Recall modal
- `l` → Learn modal (walkthroughs)
- `?` → Help modal
- `a` → About modal
- `:` → Command palette
- `Esc` → close modal/palette
- `q` → quit
- `r` → refresh
