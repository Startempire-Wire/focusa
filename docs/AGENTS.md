# AGENTS.md — Focusa Local Agent Protocol (Beads-Centered)

> This file governs agent behavior within the Focusa workspace.
> All agents MUST comply.

---

## Core Authority

- **Beads** is the authoritative task system
- **Focusa** governs focus and cognition
- Agents do not invent work

---

## Required Agent Behaviors

### Focus Discipline
- Maintain exactly one active Focus Frame
- Never switch focus implicitly
- Always bind work to a Beads issue

### Focus State Updates
- Update incrementally
- Never overwrite prior decisions
- Log contradictions explicitly

### Intuition Respect
- Do not act on intuition signals
- Surface candidates for review only

### Reference Store Usage
- Store large outputs immediately
- Reference via handles only
- Never inline large artifacts

### Expression Discipline
- Respect deterministic structure
- Do not inject hidden instructions

---

## Forbidden Agent Actions

- Autonomous task switching
- Silent memory mutation
- Bypassing Focus Gate
- Editing archived frames
- Acting without Beads backing

---

## Beads Commands (Required)

Agents MUST use documented Beads commands (`bd`) only.

### Common Commands
- `bd new`
- `bd list`
- `bd show`
- `bd next`
- `bd done`
- `bd block`
- `bd log`

If work is not tracked in Beads, it does not exist.

---

## Failure Handling

On confusion or ambiguity:
1. Pause
2. Surface candidate
3. Await instruction

---

## Final Rule

> **Meaning lives in Focus State, not in conversation.**

Agents that violate this invariant are non-compliant.
