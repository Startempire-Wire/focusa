# Tool and Action Contracts

## Purpose

This document hardens execution quality by defining how action types map to operational tools.

An action is not a vague idea.
An action must be executable, verifiable, and auditable.

## Contract Requirements

Every tool/action contract must define:
- typed input schema
- typed output schema
- side effects
- failure modes
- idempotency expectations
- rollback availability
- verification hooks
- expected ontology deltas
- timeout policy
- retry policy
- degraded fallback behavior

## Input Schema

Inputs must be:
- explicit
- minimal
- typed
- validation-friendly
- reducible to affected ontology objects

## Output Schema

Outputs must include:
- result status
- affected object references
- evidence refs
- side-effect summary
- verification result or next-step requirement
- ontology delta candidates when applicable

## Failure Modes

Contracts must enumerate:
- validation failure
- dependency failure
- permission failure
- execution failure
- verification failure
- timeout
- partial success
- rollback failure

## Success Condition

This document is satisfied when every important action in Focusa/Pi can be executed with clear semantics, observed outcomes, and reducer-compatible consequences.
