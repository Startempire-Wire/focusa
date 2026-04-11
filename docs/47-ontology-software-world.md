# Ontology Software World

## Purpose

This document defines the minimum typed world Focusa must model to support software-building cognition.

The goal is enough structure to support:
- continuity
- bounded working sets
- typed action selection
- constraint-aware execution
- re-entry after interruption

## Code World

### Repo
Required properties:
- id
- name
- root_path
- vcs_type
- default_branch

### Package
Required properties:
- id
- repo_id
- name
- package_type
- path

### Module
Required properties:
- id
- package_id
- name
- path
- language

### File
Required properties:
- id
- path
- file_type
- language

### Symbol
Required properties:
- id
- file_id
- symbol_name
- symbol_kind

### Route
Required properties:
- id
- path
- route_kind
- package_id

### Endpoint
Required properties:
- id
- path_or_signature
- method_or_transport
- package_id

### Schema
Required properties:
- id
- schema_name
- storage_kind

### Migration
Required properties:
- id
- path
- schema_targets

### Dependency
Required properties:
- id
- name
- version
- dependency_kind

### Test
Required properties:
- id
- path
- test_kind

### Environment
Required properties:
- id
- name
- environment_kind

## Work World

### Task
Required properties:
- id
- title
- status
- priority

### Bug
Required properties:
- id
- title
- severity
- status

### Feature
Required properties:
- id
- title
- status

### Decision
Required properties:
- id
- statement
- decision_kind
- status

### Convention
Required properties:
- id
- rule_text
- convention_kind
- status

### Constraint
Required properties:
- id
- rule_text
- scope
- enforcement_level

### Risk
Required properties:
- id
- title
- severity
- status

### Milestone
Required properties:
- id
- title
- status

## Mission World

### Goal
Required properties:
- id
- title
- objective
- status

### Subgoal
Required properties:
- id
- title
- status

### ActiveFocus
Required properties:
- id
- title
- frame_id
- status

### OpenLoop
Required properties:
- id
- statement
- urgency
- status

### AcceptanceCriterion
Required properties:
- id
- text
- status

## Execution World

### Patch
Required properties:
- id
- patch_ref
- timestamp

### Diff
Required properties:
- id
- diff_ref
- timestamp

### Failure
Required properties:
- id
- failure_kind
- timestamp
- status

### Verification
Required properties:
- id
- method
- result
- timestamp

### Artifact
Required properties:
- id
- handle
- artifact_kind
- status

## Design Laws

1. Every object type must support stable identity.
2. Every long-lived object must carry provenance and freshness.
3. Every object must have bounded, enumerated links.
4. Every object type must be usable in working-set construction.
5. No object type exists unless it improves software-work cognition.
