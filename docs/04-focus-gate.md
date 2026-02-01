# docs/04-focus-gate.md — Focus Gate (MVP)

## Purpose

The **Focus Gate** is the conscious filter that determines which potential concerns are allowed to surface into awareness.

It mediates between:
- the **Intuition Engine** (subconscious signals)
- the **Focus Stack** (active attention)

The Focus Gate is **advisory only**. It never switches focus automatically.

---

## Core Invariants

1. The Focus Gate never mutates Focus State or Focus Stack
2. The Focus Gate never triggers actions
3. The Focus Gate only surfaces *candidates*
4. All surfaced items are explainable
5. Decay and pressure are deterministic

---

## Candidate Model

A **Candidate** represents a potential concern that may deserve attention.

### Candidate Fields
- candidate_id
- source (intuition signal type)
- related_frame_id (optional)
- description
- pressure (float 0.0–1.0)
- age_ms
- pinned (bool)
- last_updated

---

## Pressure & Decay

### Pressure
Pressure represents accumulated salience.

Sources include:
- repetition count
- elapsed time
- severity weighting
- explicit pinning

Pressure **increases** when:
- similar signals recur
- time passes without resolution

### Decay
Pressure **decays** when:
- candidate is ignored
- candidate is suppressed
- related frame completes

Decay is:
- monotonic
- bounded
- deterministic

---

## Pinning

Pinned candidates:
- bypass decay
- persist across sessions
- must be explicitly unpinned

Pinning does **not** force focus changes.

---

## Suppression

Candidates may be suppressed:
- temporarily
- permanently
- per session

Suppression:
- reduces pressure to zero
- retains audit trail

---

## Surfacing Rules

A candidate is surfaced when:
- pressure exceeds threshold
- not suppressed
- not already visible

Surfacing produces:
- a CLI/UI notification
- an event log entry

---

## Interfaces

### Input
- Intuition Engine signals

### Output
- surfaced candidate list
- events

---

## Forbidden Behaviors

- Auto-switching Focus Stack
- Modifying Focus State
- Injecting content into prompts
- Silent escalation

---

## Acceptance Criteria

- No unexpected focus changes
- All surfaced items traceable
- Pressure behavior predictable
- Pinning behaves deterministically

---

## Summary

The Focus Gate ensures that **only meaningful concerns reach awareness**, without disrupting focus or autonomy.
