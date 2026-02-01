# docs/13-autonomy-ui.md — Autonomy Visualization (CLI + Menubar)

## Purpose

This document specifies how Focusa **visualizes autonomous capability** in a way that is:
- calm
- truthful
- verifiable
- non-manipulative
- beautiful

The UI must never *sell* autonomy — it must *show evidence*.

---

## Core Visual Metaphor

Autonomy is represented as a **halo of earned capability** around the active Focus Bubble.

- Inner state = current focus
- Outer halo = earned trust
- Texture = confidence
- Motion = stability over time

---

## CLI Interface (Authoritative)

### `focusa score now`
Displays:
- Autonomy Level (AL)
- ARI (0–100)
- Confidence band
- Top positive contributors
- Top penalties

---

### `focusa score explain --run <id>`
Shows:
- contributing Beads tasks
- event IDs
- penalties applied
- normalization factors

---

### `focusa autonomy status`
Displays:
- current AL
- granted scope
- TTL / expiry
- last promotion recommendation

---

### `focusa autonomy recommend`
Shows:
- whether promotion is justified
- why / why not
- missing evidence (e.g., sample size)

---

### `focusa autonomy grant`
Explicit human action.

```
focusa autonomy grant \
  --level 2 \
  --scope ./repo \
  --ttl 72h
```

All grants are logged and reversible.

---

## Menubar UI — Visual Spec

### Focus Bubble
- unchanged from core UI
- always central
- represents active Focus Frame

---

### Autonomy Halo

#### Geometry
- Radius proportional to Autonomy Level
- Continuous ring, not segmented

#### Appearance
- Color: grayscale → light navy accent
- Opacity: ARI (higher = clearer)
- Blur: confidence (low confidence = more diffuse)

#### Motion
- Stable if ARI rising
- Subtle wobble if ARI volatile
- No pulsing unless promotion-ready

---

### Promotion-Ready Indicator

When criteria met:
- subtle navy shimmer on halo
- no notification
- visible only on hover or inspection

---

## Timeline View (Popover)

A vertical **growth ribbon** showing:
- ARI over time
- markers for:
  - promotions
  - regressions
  - major failures

Colorless by default.
Navy accents only for milestones.

---

## Evidence Inspection

Every visual element must be inspectable:

- Click halo → list recent runs
- Click run → score breakdown
- Click penalty → event references

No “black box” visuals.

---

## Accessibility & Safety

- Motion reduction supported
- No badges, counts, or alerts
- No gamification language
- No “levels unlocked” messaging

---

## Forbidden UI Behaviors

- Celebratory animations
- Scores without evidence
- Automatic promotion actions
- Leaderboards
- Competitive framing

---

## Acceptance Criteria

- Users understand *why* autonomy exists
- Users can audit decisions
- UI never pressures escalation
- Visuals feel calm and alive
- CLI alone is sufficient

---

## Summary

Focusa visualizes autonomy as **earned stability**, not power — making trust visible without demanding it.
