# Ontology Expression and Proxy Integration

## Purpose

This document defines how ontology state becomes useful to the model.

The ontology must improve action quality by shaping the context that the model sees.

## Core Principle

Expression consumes **ontology slices**.

Proxy/harness injection must provide:
- the current mission
- the current bounded working world
- applicable decisions and constraints
- recent verified changes
- unresolved blockers

It must not provide:
- the full ontology
- broad unbounded history
- raw reducer state blobs

## Slice Composition Order

1. mission
2. active focus / active frame thesis
3. active working set
4. applicable constraints
5. recent relevant decisions
6. unresolved blockers and open loops
7. recent verified deltas
8. required artifact handles

## Operator-first slice assembly

Expression and proxy injection must be downstream of operator-intent interpretation.

The system must not assemble a slice solely from:
- active frame
- existing working set
- daemon state
- recent metacognitive overlays

Instead it must first classify the operator’s newest input and then build the smallest relevant ontology/focus slice.

### Rule
Operator-intent classification precedes slice assembly.

### Prohibition
No always-on full focus-state injection block may be treated as the default expression payload.

## Success Condition

This document is satisfied when ontology materially improves context quality without becoming a token-heavy, noisy, or echo-prone blob.
