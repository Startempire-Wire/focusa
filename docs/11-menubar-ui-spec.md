# docs/11-menubar-ui-spec.md — Focusa Menubar UI (MVP)

## Purpose

The menubar UI provides **ambient cognitive awareness** without interrupting work.

It must also make **multi-device sync** legible without becoming a control surface.
See: `docs/43-multi-device-sync.md` and the device pairing architecture in
[`docs/53-focusa-device-pairing-spec.md`](53-focusa-device-pairing-spec.md).

### Device pairing surface (focusa-ui0y)

The first-run menubar pairing surface is **Apple-like and three-party by default**
(see [§2.0 of the pairing spec](53-focusa-device-pairing-spec.md#20-primary-model--three-party-portable-phone-pwa-mediation)):

```text
Connect to Focusa

[ Mac-generated QR handoff offer ]

Scan with your phone
```

The phone PWA is the operator mediator: it scans the Mac QR, derives the current VPS
origin from `window.location.origin`, approves the Mac, and the VPS mints the token.
The Mac stores server URL + token indefinitely until the operator deliberately disconnects.

Visible first-run UI rules:

- QR card first; no wall of text.
- No server URL, public pairing URL, device name, CLI command, or device list on the primary screen.
- `Copy errors` must be available as a small secondary action.
- Manual URL/code/CLI/localhost/tunnel options live under **Advanced** only.

Fallback modes remain available after Advanced is opened
(see [§3 of the pairing spec](53-focusa-device-pairing-spec.md#3-handoff-modes)):

- **Mode A (CLI fallback):** display the `FOCUS-XXXX-XXXX` code and `on_your_vps_run` command.
- **Mode B (server-generated QR fallback):** render a QR encoding `pair_url` after the Mac already knows the VPS URL.
- **Mode C (QR + VPS browser fallback):** same QR, but operator uses a kiosk/VPS browser.

The QR is rendered using a tiny library (`qrcode` npm, ~20KB) and is sized
to be scannable at standard phone distance (≥ 200×200px, with 4-module quiet zone).

When `status=completed` from the polling loop, the panel transitions to a
single green checkmark + the device name. No celebratory animation; ambient only.

It is:
- calm
- organic
- non-demanding
- glanceable
- never modal

The UI **never becomes the primary interface**.

---

## Multi-Device Sync (Local-first)

Focusa supports multiple machines (e.g. MacBook + VPS) with **bidirectional sync**.

Menubar must make sync legible while staying calm:
- show local daemon status
- show configured peers and last sync time
- show backlog + errors (no alerts; only ambient indicators)
- show per-thread ownership (owner machine) and local attachment role
- show proposal count when contention/conflicts exist

Menubar must NOT:
- silently merge cognitive state
- auto-transfer thread ownership
- auto-resolve proposals

Authoritative policy: `docs/43-multi-device-sync.md`

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
