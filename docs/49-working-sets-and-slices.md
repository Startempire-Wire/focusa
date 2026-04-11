# Working Sets and Slices

## Purpose

This document defines the bounded object sets that Pi and Focusa should actually think within.

A developer does not think with the entire project at once.
A developer thinks with a bounded, current working world.

## Core Principle

The model should consume **ontology slices built from working sets**, not broad raw history and not the entire ontology.

## Working Set Types

### ActiveMissionSet
Contains:
- current goal/subgoal
- active task(s)
- active focus object(s)
- applicable decisions
- applicable constraints
- highest-relevance modules/files/routes/tests
- current blockers/open loops

### DebuggingSet
Contains:
- relevant failures
- suspect modules/files
- related tests
- recent diffs
- linked decisions/constraints
- recent verification results

### RefactorSet
Contains:
- target module(s)
- affected dependencies
- applicable conventions
- relevant tests
- recent decisions and risks

### RegressionSet
Contains:
- verification targets
- recent failures
- linked risks
- linked tests
- impacted modules/routes/endpoints

### ArchitectureSet
Contains:
- affected packages/modules
- conventions and constraints
- relevant decisions
- risks
- dependencies and interfaces

## Membership Classes

Every working-set member must be one of:
- pinned
- deterministic
- verified
- inferred
- provisional

## Working Set Bounds

Every set must define:
- maximum object count
- maximum artifact handle count
- maximum historical delta count
- maximum decision/constraint count per slice

## Refresh Triggers

Working sets should refresh on:
- active frame change
- goal/subgoal change
- accepted ontology delta
- failure signal
- verification result
- action-intent completion
- user pin/unpin
- session resume
- explicit refresh request

## Success Condition

Working sets are successful when Pi can act with a bounded, relevant, current world without needing broad raw history to stay coherent.
