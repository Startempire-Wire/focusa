# docs/11-menubar-ui-spec.md — Focusa Menubar UI (MVP)

## Purpose

The menubar UI provides **ambient cognitive awareness** without interrupting work.

It is:
- calm
- organic
- non-demanding
- glanceable
- never modal

The UI **never becomes the primary interface**.

---

## Design Principles

1. **Awareness, not control**
2. **Organic motion**
3. **Bottom-to-top emergence**
4. **Focus brightens, background fades**
5. **Nothing demands attention**

---

## Visual Language (Locked)

### Color
- Background: white / off-white
- Primary outline: charcoal / grayscale
- Accent: light navy
- Focused elements: mid-gray (never dark)
- Background elements: lighter by scale

### Motion
- Cloud-like drift
- No sharp linear motion
- Focus rises gently
- Resolved items fade upward and out

---

## Menubar Icon

### States
| State | Visual |
|---|---|
| Idle | Soft outline circle |
| Focused | Filled mid-gray |
| Candidates | Subtle pulse |
| Error | Temporary dark ring |

No badges.  
No numbers.

---

## Primary View (Default)

### Focus Bubble (Center)

Represents **current Focus Frame**.

- Cloud-like shape
- Slight inner glow
- Title shown on hover
- Always centered

---

### Background Thought Clouds

Represent:
- inactive Focus Frames
- pinned candidates
- archived context

Behavior:
- Drift slowly
- Fade with distance
- Never overlap focused bubble

---

## Intuition Visualization

### Intuition Pulses

- Soft concentric ripples
- Originate below view
- Drift upward
- Fade unless gated

These **never interrupt**.

---

## Focus Gate Panel (On Click)

Opens a **small vertical panel**:

- Lists surfaced candidates
- Shows pressure as opacity
- Pin / suppress actions only
- No “switch focus” button

---

## Reference Peek

On hover:
- shows artifact summary
- no content load
- click opens explicit rehydration view

---

## Interaction Rules

- No keyboard focus stealing
- No notifications
- No auto-open
- All actions reversible

---

## Update Frequency

| Element | Rate |
|---|---|
| Focus State | On change |
| Intuition pulses | ≤1/sec |
| Gate updates | On surfacing |
| Motion | 60fps CSS |

---

## Accessibility

- Motion can be reduced
- High contrast mode supported
- All info available via CLI

---

## Forbidden UI Behaviors

- Modal dialogs
- Task switching
- Editing Focus State
- Acting without confirmation
- Auto focus change

---

## MVP Acceptance Criteria

- UI never distracts
- Focus is visually obvious
- Intuition feels alive but subtle
- No measurable lag
- CLI alone remains sufficient

---

## Summary

The menubar UI is **a window into cognition**, not a control surface.

It makes the invisible visible — gently.
