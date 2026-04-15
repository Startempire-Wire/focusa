# Doc 78 Autonomy Overlap Review — 2026-04-13

Purpose:
- stop doc 78 from being fake-covered by older autonomy/governance/proposal work
- distinguish reusable substrate from true remaining doc-78 implementation

## Current overlap sources

Visible overlap clusters:
- proposal governance tests and routes
- autonomy views / constitution-related UI surfaces
- command-path autonomy hooks
- spec-hardening work already completed for doc 78

## Risk

Older autonomy/governance work can look like doc-78 completion even when the following remain incomplete:
- bounded secondary/background cognition call-site inventory
- truthful separation of heuristic vs model-backed secondary cognition
- persistent autonomy state bounded by operator priority and routing rules
- trace/eval surfaces proving secondary cognition is structured rather than gibberish-prone
- blocked-work mapping against docs 51/54a/54b/56/57/67-77

## Reuse vs not-yet-done

### Reusable substrate
- proposal governance concepts
- autonomy command surfaces
- some trace/checkpoint hooks
- constitution/governance UI surfaces

### Not sufficient by themselves
- a proposal route existing
- an autonomy level existing
- a governance test passing
- a UI panel rendering autonomy state

These do not prove bounded secondary cognition or persistent-autonomy semantics from doc 78.

## Required doc-78 decomposition rule

Every doc-78 bead must state one of:
- **reuses existing substrate**
- **extends existing substrate**
- **blocked on prerequisite branch**
- **new implementation surface**

No doc-78 bead should rely on generic “autonomy/governance exists” wording.

## Immediate decomposition consequences

Doc 78 needs explicit branches for:
1. secondary/background cognition call-site inventory
2. heuristic-vs-model-backed path classification
3. operator-priority bounds on persistent autonomy behavior
4. trace/eval proof surfaces for secondary cognition quality
5. blocked-work map against routing/shared-substrate/governance branches
