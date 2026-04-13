# Tool and Action Contracts

## Purpose

This document hardens execution quality by defining how action types map to operational tools.

An action is not a vague idea.
An action must be executable, verifiable, and auditable.

SPEC 55 governs the contract surface for Pi tools, Focusa bridge tools, and Focusa command submission.
The goal is not merely "a tool exists".
The goal is that every tool call has predictable semantics under success, retry, failure, degradation, and recovery.

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

The caller must be able to answer all of the following before execution:
- what object or surface will change?
- what evidence will prove the change occurred?
- can the action be retried safely?
- if retried, what prevents duplication or corruption?

## Output Schema

Outputs must include:
- result status
- affected object references
- evidence refs
- side-effect summary
- verification result or next-step requirement
- ontology delta candidates when applicable

Output schemas may be tool-specific, but they must always distinguish:
- accepted vs completed
- completed vs partially completed
- canonical vs degraded fallback result
- local shadow update vs confirmed remote persistence

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

### Failure Taxonomy

| Failure mode | Meaning | Retry default | Required evidence |
|---|---|---|---|
| Validation failure | Input rejected before side effects | Do not retry unchanged input | Rejected field or violated rule |
| Dependency failure | Required daemon/service/tool unavailable | Retry only after dependency health change | Health check or missing dependency signal |
| Permission failure | Caller lacks authority or filesystem/network access | Do not blind-retry | Permission boundary identified |
| Execution failure | Tool started but failed before intended outcome | Retry only if action class permits | Exit/status/error payload |
| Verification failure | Action may have run, but required postcondition not proven | Retry only with dedupe/idempotency guard | Missing/failed verification hook |
| Timeout | Completion unknown before deadline | Treat as ambiguous; verify before retry | Timeout marker + last known state |
| Partial success | Some side effects happened, contract not fully satisfied | Retry only for remaining delta | Partial artifact or partial state mutation |
| Rollback failure | Recovery attempt itself could not restore baseline | Escalate; do not loop | Recovery result and residual damage |

### Ambiguous Completion Rule

Timeout and connection-loss cases are not equivalent to clean failure.
If completion is ambiguous, the caller must verify state before retrying.
If verification is impossible, the system must surface the result as unknown rather than silently assuming either success or failure.

## Idempotency Expectations

### Idempotency Classes

| Class | Definition | Retry posture |
|---|---|---|
| Strongly idempotent | Same request can be replayed without changing final state | Safe to retry automatically |
| Conditionally idempotent | Safe only when scoped by target identity, expected prior state, or dedupe key | Retry with guards only |
| Append-only non-idempotent | Repeating the request creates another record/event | Never blind-retry |
| Replace-style non-idempotent | Repeating may overwrite newer state | Retry only with operator intent or compare-and-set semantics |
| Process-executing non-idempotent | Repeating may rerun commands or duplicate external effects | Never automatic unless wrapped by external idempotency |

### Default Tool Classes

| Tool/action class | Idempotency class | Notes |
|---|---|---|
| `read`, `grep`, `find`, pure status queries | Strongly idempotent | Read-only; no side effects |
| `focusa_intent`, `focusa_current_focus` | Conditionally idempotent | Replace semantic on a known slot |
| `focusa_next_step`, `focusa_open_question`, `focusa_recent_result`, `focusa_note` | Append-only non-idempotent | Replays duplicate slot entries |
| `focusa_decide`, `focusa_constraint`, `focusa_failure` | Append-only non-idempotent unless deduped by caller | Semantic duplicates are still duplicates |
| `focusa_scratch` | Append-only non-idempotent | Each note is another scratch entry |
| `write` | Replace-style non-idempotent | Replays overwrite file content |
| `edit` | Conditionally idempotent | Safe only while expected old text still matches |
| `bash`, `mcp`, external command execution | Process-executing non-idempotent by default | Depends on wrapped command semantics |
| `/commands/submit` with explicit `idempotency_key` | Conditionally idempotent | Key must scope one intended command instance |

### Idempotency Requirements

1. Read-only actions must remain side-effect free.
2. Append-style actions must be documented as non-idempotent unless an explicit dedupe key exists.
3. Replace-style actions must define last-write-wins vs compare-and-set semantics.
4. Execution-style actions must assume replay is dangerous unless proven otherwise.
5. Any automatic retry path for non-read-only work must name its dedupe guard.

## Retry Policy

Retry policy must be tied to idempotency class.

| Action class | Retry policy |
|---|---|
| Strongly idempotent | Automatic retry allowed on transient transport/dependency failure |
| Conditionally idempotent | Retry only with dedupe key, expected old state, or verification gate |
| Append-only non-idempotent | No automatic retry; verify and require explicit re-issue |
| Replace-style non-idempotent | No blind retry; verify current state first |
| Process-executing non-idempotent | No automatic retry unless wrapped by external transactional/idempotent control |

## Verification Hooks

Every contract must declare how completion is verified.
Accepted verification patterns:
- direct return status with typed success payload
- post-read of changed state
- artifact existence/content check
- reducer-visible ontology delta
- explicit acknowledgement from remote command processor

Verification must occur at the right layer.
For example:
- local shadow update is not proof of remote Focusa persistence
- command acceptance is not proof of downstream completion
- file write return is not proof that a separate service consumed the file

## Rollback and Recovery

Each contract must declare one of:
- rollback supported
- compensating action supported
- no rollback; forward recovery only

The absence of rollback is acceptable only when the contract explicitly names:
- what residual state may remain
- how operators detect that residual state
- what forward recovery path exists

## Degraded Fallback Behavior

A degraded path must be marked non-canonical when it bypasses the primary authority.

Examples:
- Focusa unavailable → local Pi fallback compaction may proceed, but result is non-canonical
- local scratch write succeeds while Focusa write fails → scratchpad is evidence, not Focus State persistence
- command accepted by bridge but not verified in Focusa → status remains accepted/unknown, not completed

## Required Documentation Shape per Tool

Every important tool spec must include a concise table with at least:
- action name
- target surface
- side effects
- idempotency class
- retry rule
- verification hook
- degraded fallback
- rollback/recovery note

## Worked Focusa/Pi Examples

### Example A — `focusa_decide`
- target surface: Focus State decisions slot
- side effect: append one crystallized architectural decision
- failure risk: validation rejection, Focusa unavailable, remote update failure
- idempotency: append-only non-idempotent
- retry rule: do not blind-retry after ambiguous transport failure; verify whether the decision was already recorded
- verification: successful `pushDelta`/Focusa update acknowledgement plus subsequent frame read when needed
- rollback: none; forward recovery by recording a corrective decision or resetting frame intentionally

### Example B — `edit`
- target surface: file content matching an expected old-text region
- side effect: in-place file mutation
- failure risk: old text mismatch, overlapping edits, permission failure, partial human misunderstanding of effect
- idempotency: conditional; replay is safe only while the expected old text still exists
- retry rule: recompute against current file content before retry
- verification: read-back diff or exact file content check
- rollback: possible only if prior content is known

### Example C — Focusa command submit: compact
- target surface: Focusa command processor
- side effect: queue or trigger compaction
- failure risk: daemon offline, acceptance without completion proof, timeout, fallback to local compact
- idempotency: conditional via `idempotency_key`
- retry rule: reuse the same intent guard when retrying the same command instance; do not generate duplicate compactions blindly
- verification: command acknowledgement plus resulting post-compact state/evidence
- degraded fallback: local compact allowed only when explicitly marked non-canonical

## Success Condition

This document is satisfied when every important action in Focusa/Pi can be executed with clear semantics, observed outcomes, reducer-compatible consequences, and documented retry/idempotency behavior.
