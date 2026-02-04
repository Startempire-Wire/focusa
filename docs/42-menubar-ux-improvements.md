# docs/42-menubar-ux-improvements.md — Menubar UX Improvement Spec

> **Status:** Draft — awaiting hands-on testing of v0.2.0 before implementation  
> **Prerequisite:** User must test current menubar app and confirm/adjust priorities  
> **Approach:** Iterative improvement of existing UI, not a redesign

---

## Research Summary

### Sources Consulted

**Calm Technology (foundational theory):**
- Mark Weiser & John Seely Brown: "Technology that informs but doesn't demand our focus or attention"
- Amber Case's 8 principles of calm technology: (1) require smallest possible amount of attention, (2) inform and create calm, (3) move easily from periphery to center and back, (4) amplify the best of technology and the best of humanity
- Key insight: *the current cloud/bubble metaphor is aligned with calm technology principles* — the problem is execution, not concept

**Ambient Display Research (academic):**
- Shelton & Nesbitt (2016): ambient displays "minimise stresses on cognitive load by delivering information in a peripheral, unfocussed manner"
- Cognitive load trade-off: there is a measurable penalty when ambient displays become too information-dense; the sweet spot is *one glanceable metric + progressive disclosure*
- The "aesthetic awareness display" pattern: information encoded as visual properties (color, opacity, motion) rather than text — exactly what our bubble/pulse model does

**Apple Human Interface Guidelines (platform rules):**
- Menu bar extras: "In general, display a menu, not a popover, when the user clicks your menu bar extra. Unless the app functionality you want to expose is too complex for a menu, avoid presenting it in a popover." — Focusa qualifies as "too complex for a menu" ✓
- Popovers: "A popover is a transient view that appears above other content." Should dismiss when clicking outside ✓
- Dark mode: apps should respect `prefers-color-scheme` — Apple strongly recommends supporting both appearances
- Window vibrancy: `NSVisualEffectView` materials (`under-window`, `sidebar`, `popover`) give native translucent feel

**macOS Menubar App Patterns (competitive analysis):**
- **AnyBar** (tonsky): colored dot as the entire interface — proves a single status indicator is powerful
- **TomatoBar** (ivoronin): pomodoro timer, minimal menu bar icon changes color with state
- **Session** (focus timer): "small status bar" praised by users — shows timer + task name in bar
- **Ice** (jordanbaird): menu bar manager, praised for clean popover design
- **Raycast**: compact popover, keyboard-first, fuzzy search — but this is a launcher, not ambient display

**UX Patterns (NN/g, IxDF, Contentsquare):**
- Progressive disclosure: "revealing the right information, at the right time" — hover/click to expand detail
- Cognitive load minimization: "only include the most important information; use size, color, contrast, and placement to establish clear visual hierarchy"
- Tooltips as progressive disclosure: "reduce cognitive load, prevent errors, make your application feel sophisticated"
- False bottom anti-pattern: "provide visual signifiers that indicate users can scroll or reveal more content"

**Tauri v2 Capabilities:**
- `window-vibrancy` crate: `apply_vibrancy(&window, NSVisualEffectMaterial::Popover)` gives native macOS frosted glass
- `prefers-color-scheme` CSS media query works in Tauri webview — respects system dark/light setting
- Transparent windows: already configured in `tauri.conf.json` (`"transparent": true`)
- Tauri bug #5802: window theme may not propagate to webview `prefers-color-scheme` — workaround: detect via JS `window.matchMedia`

---

## What to Keep (Working Design Decisions)

These aspects of the current UI are *correctly aligned* with both the spec and calm technology research. Do not change them:

| Element | Why It's Right |
|---------|---------------|
| **Cloud/bubble metaphor** | Encodes cognitive state as visual properties (opacity, position, motion) rather than demanding text parsing — matches ambient display research |
| **Central focus bubble** | Single focal point; calm technology principle: "one thing at the center of attention" |
| **Drifting background clouds** | Peripheral awareness of inactive frames without demanding attention — textbook ambient display pattern |
| **Concentric intuition ripples** | Bottom-to-top emergence matches spec; ripple motion is organic and non-demanding |
| **Gate panel on click** | Progressive disclosure — detail revealed only when sought. Popover pattern is correct per Apple HIG for complex status |
| **Settings panel** | Local/Tailscale presets, test connection, setup guide — all genuinely useful |
| **2-second polling** | Non-aggressive update frequency; spec says "Focus State: on change" but polling is acceptable for MVP |
| **320×420 window size** | Within Apple's recommended popover size range; compact enough for menubar |
| **No notifications, no modals** | Spec-mandated; confirmed by calm technology principles |
| **CSS motion variables** | `--transition-gentle`, `--transition-drift`, `--transition-fade` — organic, cloud-like |
| **`prefers-reduced-motion` support** | Already implemented; accessibility requirement |
| **`prefers-contrast: high` support** | Already implemented; accessibility requirement |

---

## Issues to Fix (Prioritized)

### P0 — Broken or Misleading

#### 1. No Dark Mode

**Problem:** Light-only theme (`--bg: #fafafa`). Developers overwhelmingly use dark macOS. The app will appear as a jarring white rectangle in a dark environment.

**Evidence:** Apple HIG: "Always support both light and dark appearances." Every competitive menubar app supports system appearance detection.

**Fix:** Add `@media (prefers-color-scheme: dark)` block in `tokens.css` with dark color values. Use the existing TUI's charcoal/navy palette (`#1a1b2e` background, `#e8c77b` accent) for brand consistency across CLI and desktop.

**Scope:** `tokens.css` only — all components already use CSS variables, so they inherit automatically. Zero component changes.

**Dark palette (proposed):**
```css
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #1a1b2e;
    --bg-panel: #232440;
    --fg: #d4d4d8;
    --fg-dim: #9090a0;
    --fg-muted: #606070;
    --accent: #e8c77b;         /* golden — matches app icon */
    --accent-soft: #c4a060;
    --active: #e8c77b;
    --active-glow: rgba(232, 199, 123, 0.12);
    --border: #3a3b52;
    --warn: #e8c77b;
    --error: #e85050;
    --success: #50c878;
    --shadow-subtle: 0 1px 3px rgba(0, 0, 0, 0.2);
    --shadow-panel: 0 2px 8px rgba(0, 0, 0, 0.3);
  }
}
```

#### 2. Connection Status Invisible

**Problem:** The only way to know if the app is connected to the daemon is to open Settings. The main view gives zero indication.

**Evidence:** AnyBar (12,000+ GitHub stars) proves a single colored dot is the most effective status indicator for menubar apps. Ethernet Status, Menu Meters, and every VPN client use the same pattern.

**Fix:** Add a small (6px) status dot to the focus bubble or to a thin header strip. Three states:
- **Green (`--success`):** Connected, daemon responding
- **Amber (`--warn`):** Connected but no active session
- **Red/dim (`--error`):** Disconnected / daemon unreachable

**Scope:** Small addition to `FocusBubble.svelte` — a `<div class="status-dot">` positioned at the top-right of the bubble, colored via `focusStore.connected`.

#### 3. ReferencePeek Is `display: none`

**Problem:** Component exists but renders nothing. It's dead UI taking up a component slot.

**Fix:** Either:
- **(a)** Remove the component entirely from the page (honest — it's not implemented)
- **(b)** Show minimal reference info: artifact count from `reference_index.handles.length` in the state dump, displayed as a small label near the bubble (e.g., "3 refs")

**Recommendation:** Option (a) for now. Add it back when the feature is real. Dead UI is worse than no UI.

---

### P1 — Usability Gaps

#### 4. Bubble Requires Hover to Communicate Anything

**Problem:** The 64px circle shows no text until hovered. First-time users see a filled or unfilled circle and don't know what it means. The hover tooltip appears *below* the bubble — if the bubble is centered, the tooltip may be clipped or hard to reach on small trackpads.

**Evidence:** Calm technology principle: "Technology should inform *and* create calm." Current bubble informs nothing without interaction. Ambient display research: "one glanceable metric" is the sweet spot.

**Fix:** Show a short label below the bubble at all times — the active frame's `intent`, truncated to ~25 characters with ellipsis. This is still calm (small, muted text) but makes the bubble *glanceable* without interaction.

**Implementation:**
```svelte
<!-- Below the bubble, always visible -->
{#if frame}
  <div class="bubble-label">{frame.intent.slice(0, 25)}{frame.intent.length > 25 ? '…' : ''}</div>
{:else}
  <div class="bubble-label idle-label">No active focus</div>
{/if}
```

Style: `font-size: var(--font-size-sm)`, `color: var(--fg-dim)`, centered below bubble. The hover tooltip still exists for showing the full intent + `current_state` — progressive disclosure is preserved.

#### 5. Background Clouds Overlap at 3+ Frames

**Problem:** Position is computed as `left: {20 + (i * 30) % 80}%` and `top: {15 + (i * 25) % 70}%`. At 3 frames: positions are (20%, 15%), (50%, 40%), (80%, 65%) — OK. At 4: (20%, 15%), (50%, 40%), (80%, 65%), (40%, 20%) — cloud 0 and cloud 3 are close. At 5+: guaranteed overlaps.

**Fix:** Use a deterministic non-overlapping layout. Two approaches:
- **(a)** Arc/ring layout: place clouds on an elliptical arc around the center bubble, evenly spaced by angle. No overlap is possible because the arc radius ensures minimum distance.
- **(b)** Grid-snap: divide the available area (excluding center 80px) into a grid, place one cloud per cell.

**Recommendation:** Option (a) — the arc layout is more organic and matches the "thought cloud" metaphor. Clouds orbit the focus at varying distances.

**Implementation sketch:**
```js
// Place clouds on an ellipse around center
const angleStep = (2 * Math.PI) / frames.length;
const rx = 100; // horizontal radius (px)
const ry = 70;  // vertical radius (px)
// For frame i:
const angle = angleStep * i - Math.PI / 2; // start from top
const x = 50 + (rx * Math.cos(angle)) / 320 * 100; // % of container width
const y = 50 + (ry * Math.sin(angle)) / 420 * 100; // % of container height
```

#### 6. Gate Panel Has No Actions

**Problem:** The spec explicitly says "Pin / suppress actions only" for the gate panel. Current implementation shows candidates but has zero action buttons. The panel is read-only.

**Fix:** Add two small action buttons per candidate, visible on hover (progressive disclosure):
- **📌 Pin** — `POST /v1/focus-gate/pin` with `{ candidate_id }`
- **🔇 Suppress** — `POST /v1/focus-gate/suppress` with `{ candidate_id, scope: "session" }`

These are fire-and-forget API calls. The next 2-second poll will reflect the updated state.

**Implementation:** Add to `GatePanel.svelte` inside each `.candidate` div:
```svelte
<div class="candidate-actions">
  {#if !candidate.pinned}
    <button class="action-btn" onclick={() => pinCandidate(candidate.id)} title="Pin">📌</button>
  {/if}
  <button class="action-btn" onclick={() => suppressCandidate(candidate.id)} title="Suppress">🔇</button>
</div>
```

Style: `.candidate-actions` is `opacity: 0` by default, `opacity: 1` on `.candidate:hover`. Buttons are 20px, no border, transparent background.

The `pinCandidate` and `suppressCandidate` functions call the API:
```ts
async function pinCandidate(id: string) {
  const base = localStorage.getItem('focusa_api_url') || 'http://127.0.0.1:8787';
  await fetch(`${base}/v1/focus-gate/pin`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ candidate_id: id }),
  });
}
```

---

### P2 — Polish & Native Feel

#### 7. No Window Vibrancy

**Problem:** The popover is an opaque white rectangle. Native macOS popovers have a translucent frosted-glass effect using `NSVisualEffectView`.

**Fix:** Add `window-vibrancy` crate to Tauri backend. Apply `NSVisualEffectMaterial::Popover` on setup. The CSS background becomes `transparent` / semi-transparent, letting the native blur show through.

**Implementation (Rust side):**
```toml
# apps/menubar/src-tauri/Cargo.toml
[dependencies]
window-vibrancy = "0.5"
```

```rust
// main.rs setup
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

tauri::Builder::default()
    .setup(|app| {
        let window = app.get_webview_window("main").unwrap();
        #[cfg(target_os = "macos")]
        apply_vibrancy(&window, NSVisualEffectMaterial::Popover, None, None)
            .expect("Failed to apply vibrancy");
        Ok(())
    })
```

**CSS side:** Change `--bg` to `rgba(250, 250, 250, 0.8)` in light mode, `rgba(26, 27, 46, 0.8)` in dark mode. The native blur fills in behind.

**Fallback:** On Linux/Windows (no vibrancy), the 0.8 alpha gives a subtle transparency. On older macOS, `window-vibrancy` falls back gracefully.

#### 8. Tray Icon Doesn't Reflect State

**Problem:** The tray icon (`icon.png`) is static. The spec defines 4 tray icon states: Idle (soft outline), Focused (filled mid-gray), Candidates (subtle pulse), Error (temporary dark ring). None are implemented.

**Fix:** Generate 3 tray icon variants at build time:
- `tray-idle.png` — outline circle (current)
- `tray-focused.png` — filled circle
- `tray-error.png` — circle with red ring

Switch icons from Rust based on daemon connection state:
```rust
// In a periodic check or on state update
let icon = if !connected { "tray-error.png" }
           else if has_active_frame { "tray-focused.png" }
           else { "tray-idle.png" };
app.tray_by_id("main").unwrap().set_icon(Some(Icon::File(icon.into())));
```

**Note:** Tray icon animation (pulse for candidates) is not feasible with static PNGs. Save for a future enhancement using `iconAsTemplate: true` + dynamic rendering.

#### 9. Settings Panel Lacks Theme Toggle

**Problem:** No way to override system theme. Some users want dark mode even on a light-mode system (or vice versa).

**Fix:** Add a 3-way theme selector to Settings: **System** (default), **Light**, **Dark**. Store preference in `localStorage('focusa_theme')`. Apply by toggling a `data-theme="dark"` attribute on `<html>`.

CSS changes:
```css
/* System-follows mode (default) */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) { /* dark vars */ }
}
/* Forced dark mode */
:root[data-theme="dark"] { /* dark vars */ }
```

#### 10. Intuition Ripples Give No Information

**Problem:** Concentric ripples expand and fade — they indicate "something is happening" but give zero detail about *what*. The signal kind and summary are never displayed.

**Fix:** Add a small text label below the ripples showing the most recent signal's summary, truncated. Appears only when signals are active. Fades out after 5 seconds of no new signals.

```svelte
{#if active && latestSignal}
  <div class="signal-label">{latestSignal.kind}: {latestSignal.summary?.slice(0, 40)}</div>
{/if}
```

**Data source:** `intuitionStore` already has `signals` array. Add a derived `latestSignal` getter:
```ts
get latestSignal() {
  return signals.length > 0 ? signals[signals.length - 1] : null;
},
```

Style: `font-size: 0.65rem`, `color: var(--fg-muted)`, `text-align: center`, `opacity` animated.

---

### P3 — Nice-to-Have (Future)

These are recorded for future iterations but should NOT be implemented before testing:

#### 11. Keyboard Shortcuts

Menubar apps benefit from keyboard-first interaction (Raycast model). Potential shortcuts:
- `Esc` — dismiss popover
- `G` — toggle gate panel
- `S` — toggle settings

Not in MVP — conflicts with the "no keyboard focus stealing" spec rule. Revisit after testing.

#### 12. Elapsed Time Display

Show how long the current focus frame has been active. Requires either:
- A `created_at` or `started_at` timestamp on `FrameRecord` (not currently exposed in the API state dump)
- Client-side tracking (inaccurate across reconnects)

Deferred until the API exposes frame timestamps.

#### 13. Stack Depth Indicator

A subtle arc or ring around the bubble showing stack depth (e.g., 3/8). Gives peripheral awareness of nesting without text. Lower priority than the text label (P1.4).

#### 14. Candidate Count Badge on Gate Trigger

The gate trigger dot pulses when candidates are surfaced. Adding a tiny number (e.g., "3") next to it would give instant count information. But this borders on "badge notification" which the spec forbids. Discuss with user.

---

## Implementation Order

If the user approves after testing:

1. **Dark mode** (P0.1) — `tokens.css` only, instant impact, zero risk
2. **Connection status dot** (P0.2) — small `FocusBubble.svelte` change
3. **Remove ReferencePeek** (P0.3) — delete dead code
4. **Bubble label** (P1.4) — small `FocusBubble.svelte` change
5. **Cloud layout fix** (P1.5) — `FocusStackView.svelte` math change
6. **Gate actions** (P1.6) — `GatePanel.svelte` + small API calls
7. **Window vibrancy** (P2.7) — Tauri backend + CSS opacity
8. **Tray icon states** (P2.8) — Rust + icon generation
9. **Theme toggle** (P2.9) — `Settings.svelte` + `tokens.css`
10. **Signal label** (P2.10) — `IntuitionPulse.svelte` + store getter

Total estimated changes: ~200 lines added, ~30 lines removed, across 6 files.

---

## Files Affected

| File | Changes |
|------|---------|
| `static/styles/tokens.css` | Dark mode palette (P0.1), vibrancy alpha (P2.7) |
| `src/lib/components/FocusBubble.svelte` | Status dot (P0.2), text label (P1.4) |
| `src/lib/components/FocusStackView.svelte` | Arc layout math (P1.5) |
| `src/lib/components/GatePanel.svelte` | Pin/suppress buttons (P1.6) |
| `src/lib/components/IntuitionPulse.svelte` | Signal label (P2.10) |
| `src/lib/components/ReferencePeek.svelte` | Delete or gut (P0.3) |
| `src/lib/components/Settings.svelte` | Theme toggle (P2.9) |
| `src/lib/stores/intuition.ts` | `latestSignal` getter (P2.10) |
| `src/routes/+page.svelte` | Remove ReferencePeek import (P0.3) |
| `apps/menubar/src-tauri/Cargo.toml` | `window-vibrancy` dep (P2.7) |
| `apps/menubar/src-tauri/src/main.rs` | Vibrancy setup + tray icon switching (P2.7, P2.8) |

---

## Design Tokens Reference (Current → Proposed)

### Light Mode (keep current, minor adjustments)
```
--bg:           #fafafa → rgba(250, 250, 250, 0.85)  (vibrancy)
--bg-panel:     #ffffff → rgba(255, 255, 255, 0.9)   (vibrancy)
All other light tokens: unchanged
```

### Dark Mode (new)
```
--bg:           rgba(26, 27, 46, 0.85)    charcoal (matches TUI)
--bg-panel:     rgba(35, 36, 64, 0.9)     slightly lighter
--fg:           #d4d4d8                    soft white
--fg-dim:       #9090a0                    muted
--fg-muted:     #606070                    very muted
--accent:       #e8c77b                    golden (matches app icon)
--accent-soft:  #c4a060                    dimmer gold
--active:       #e8c77b                    same as accent in dark
--active-glow:  rgba(232, 199, 123, 0.12) subtle gold glow
--border:       #3a3b52                    dark line
--warn:         #e8c77b                    amber
--error:        #e85050                    red
--success:      #50c878                    green
```

---

## Validation Criteria

After implementation, verify:

- [ ] Dark mode auto-detects from macOS system preference
- [ ] All text remains readable in both light and dark modes
- [ ] Connection status dot is visible within 0.5s of opening the popover
- [ ] Active focus intent is readable without any interaction
- [ ] 5 inactive frames do not visually overlap
- [ ] Pin/suppress buttons work and state updates on next poll
- [ ] Vibrancy shows native blur on macOS (degrades gracefully elsewhere)
- [ ] Tray icon changes between idle/focused/error states
- [ ] `prefers-reduced-motion: reduce` disables all animations
- [ ] `prefers-contrast: high` remains functional with dark mode
- [ ] No new forbidden UI behaviors introduced (no modals, no notifications, no auto-focus-change)
- [ ] CLI remains fully sufficient — menubar is supplementary only
- [ ] Total render time per frame < 16ms (60fps CSS)
- [ ] Polling doesn't accumulate if daemon is slow (abort previous on new tick)

---

## Open Questions (For User Testing)

1. **Bubble label vs. no label:** Does the always-visible text feel calm or cluttered?
2. **Gate actions scope:** Should suppress be "session" or offer a "permanent" option?
3. **Signal label duration:** 5s fade-out, or persistent until next signal?
4. **Arc vs. grid for clouds:** Which feels more organic with real task names?
5. **Vibrancy opacity:** 0.85 might be too transparent or not enough — needs visual testing
6. **Tray icon states:** Do 3 states suffice, or do you want a 4th for "candidates surfaced"?
7. **Theme toggle placement:** In Settings panel, or a separate icon in the header?

---

*This spec improves the existing UI with 10 targeted changes grounded in calm technology research, Apple HIG, and ambient display UX principles. No radical redesign. Every change is reversible.*
