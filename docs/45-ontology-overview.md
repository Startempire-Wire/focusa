# Focusa Ontology Overview

## Purpose

This document defines the Ontology layer for Focusa.

The Ontology is **additive in implementation** and **canonical in semantics**. It does not replace the existing Focusa runtime. It supplies the missing typed software-world model that the runtime, proxy layer, and Pi extension should consume.

Focusa already has strong primitives for:
- focus continuity
- bounded expression
- gating and salience
- reducer-governed state
- reference/artifact handling
- thread and thesis flow

What it lacks is a first-class semantic model of the software world being worked on.

The Ontology closes that gap.

## Core Thesis

Focusa should not merely preserve cognitive fragments.

Focusa should provide Pi with a **typed, bounded, interruptible working world** for software building.

That working world must define:
- what exists
- how things relate
- what actions are valid
- what currently matters
- what has been verified
- what remains uncertain

## Ownership Split

### Ontology owns
- canonical software-world semantics
- object identity and type rules
- typed relations
- typed actions
- working sets
- provenance
- verification state
- status and freshness

### Focusa runtime owns
- continuity across turns
- active focus stack and thesis flow
- salience/gate decisions
- bounded prompt assembly
- reducer authority
- persistence, replay, trace, recovery
- policy and runtime enforcement

### Pi extension owns
- harness-side integration
- consuming ontology slices
- invoking Focusa cognition tools at the correct times
- emitting proposals, action intents, failures, and observations
- never becoming a second cognitive authority

## Architectural Law

Deterministic systems classify structure.  
Background models classify ambiguity.  
Only the reducer canonizes ontology truth.

## Why Ontology Exists

Ontology exists to solve problems that focus state alone cannot solve:

- preserving a working world across interruptions
- grounding model behavior in typed reality rather than loose history
- reducing ambiguity for weaker models
- making decisions and constraints actually actionable
- providing a stable basis for working-set construction
- allowing typed actions instead of vague intent

## Canonical Truth

Ontology truth is canonical only when it has passed through the reducer path.

This means:
- parsers and extractors may produce facts or proposed deltas
- background models may produce proposals
- Pi may produce action intents and observations
- tool results may produce evidence
- only reducer-approved writes become canonical ontology state

## Success Condition

The Ontology layer is successful when it becomes the semantic spine of Focusa without destabilizing the current cognitive runtime.
