# AGENTS.md — Focusa Local Agent Protocol (Beads-Centered)

> This file governs agent behavior within the Focusa workspace.
> All agents MUST comply.

---

## Agent-KB API Default Reference

For KH/OVH/operator policy, use `agent-kb-api` first, verify freshness, retrieve exact documents after empty searches, and use local Agent KB files only as a read-only degraded fallback.

## Agent communications + GitHub 2FA documentation contract

- Communications design must identify authorized `github.com` SMS OTP retrieval/injection as the immediate release-critical use case while prohibiting ambient message access.
- Specifications must extend Spec 156 credential/MFA authority and existing privacy, adapter, audit, placement, and Veragensia lifecycle contracts rather than create a parallel secret system. OTP values are ephemeral P4 material: persist only redacted handles and value-free evidence.
- GitHub OTP is the first bounded slice. Specifications must preserve a later customer-authorized SMS API for thread listing, bounded reads, sends, and events, with capabilities and consent distinct from OTP access; no privilege widening is allowed.
- Shared CLI/API/MCP/OpenClaw contracts stay transport-neutral behind versioned connector adapters. Android/Google Messages is a bounded bootstrap; **iPhone/iOS is a concurrent urgent target**, with an explicit supported/user-consented integration decision, parity matrix, migration/portability path, real-device acceptance, and no dependency on private Apple APIs.
- Every plan must cover scoped provider/challenge/message-class authorization, injection without routine plaintext exposure, encryption, restart recovery, health, revocation/re-pairing, attribution, audit, rate limits, replay/duplicate-send defense, prompt-injection resistance, customer handoff, and zero-residue teardown. Recovery codes are out of scope and permanently forbidden.

## Core Authority

- **Beads** is the authoritative task system
- **Focusa** governs focus and cognition
- Agents do not invent work

## Public / Private Docs Boundary

Private operator docs may exist locally at `.focusa-private/`.

Agents must read `.focusa-private/INDEX.md` before touching SaaS strategy, SignalOS, commercial pricing/caps, install/purchase backend, raw proof, launch planning, or vendor/license registry work.

Agents must never commit `.focusa-private/`, raw transcripts, runtime objects, local host paths, admin URLs, customer data, or license data.

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
