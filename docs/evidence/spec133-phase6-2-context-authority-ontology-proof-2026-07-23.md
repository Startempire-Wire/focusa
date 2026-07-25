# Spec 133 Phase 6.2 — Context Cognition, Context Authority, and ontology

Date: 2026-07-23
Bead: `focusa-a6yq6.7.2`
Scope: Spec 133 §19.7–§19.12

## Context Cognition boundary

`ContextBootstrapBinding` remains explicitly advisory and now binds:

- selected and excluded context;
- risk refs;
- valid next tools;
- project, continuity, trajectory, and Workpoint refs;
- source snapshot/hash and freshness;
- explicit prohibition on canonical mutation authority.

The AgentBootstrap packet continues to carry active objects, evidence/proof gaps, blockers, exact next action, and do-not-drift constraints.

## Typed ontology bindings

`SilentSessionOntologyBindings` requires non-empty refs for all Spec 133 ontology classes:

`AgentIdentity`, `ActorInstance`, `RoleProfile`, `CapabilityProfile`, `PermissionProfile`, `Responsibility`, `HandoffBoundary`, `ExecutionContext`, `ToolSurface`, `Affordance`, `Resource`, `CostModel`, `ReliabilityProfile`, `ReversibilityProfile`, `WorkItem`, `ActionIntent`, `Blocker`, `VerificationRecord`, and `EvidenceArtifact`.

Missing refs fail bootstrap validation. The composed Focusa authority envelope carries the exact ontology projection, Context Cognition risks, and valid-next-tool refs.

## Action-specific Context Authority

`ContextAuthorityGrant` now binds:

- typed action class;
- exact action name;
- deterministic action digest;
- issue and expiry times;
- exact project, continuity, and Workpoint scope.

The generic authorization path rejects action/name/digest mismatch, future issuance, expiry, and grants lasting more than five minutes.

For AgentBootstrap-governed actions, `context_authority_action_digest` hashes the exact bootstrap packet SHA-256 plus action class and action name. A grant for restart cannot authorize release, integration, deployment, or another mutation.

Supported classes cover session launch mutation, daemon/service restart, git integration, deploy, release, database migration, destructive file operations, secret/config changes, cross-project edits, generated-code overwrite, model/trust-policy changes, and other risky mutation.

## Major transition barrier

`verify_major_transition_barrier` requires:

1. a fresh verification receipt for the exact AgentBootstrap packet;
2. a fresh Context Authority grant for the exact action class/name/digest;
3. matching ProjectIdentity, Continuity, and Workpoint.

It emits a bounded type-state grant with the exact session/run/generation, action digest, Context Authority ref, and earliest validity deadline. Project mutation uses the same action verifier in addition to writer lease, model, and workspace barriers.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run for this slice.

```bash
rustfmt --edition 2024 --check <changed Rust files>
git diff --check
```

Result: passed.

Static constructor audits found no missing Context Authority action fields, ontology bindings, Context Cognition risk/tool refs, or project-mutation action bindings.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-core silent_session_authorization -- --nocapture
cargo test -p focusa-core silent_session_bootstrap -- --nocapture
cargo test -p focusa-core silent_session_authority -- --nocapture
cargo test -p focusa-session-runner mutation_posix -- --nocapture
cargo test -p focusa-api silent_sessions -- --nocapture
cargo test -p focusa-core
cargo test -p focusa-session-runner
cargo test -p focusa-api
cargo clippy -p focusa-core -p focusa-session-runner -p focusa-api --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove wrong-action and overlong Context Authority rejection, major-transition action isolation, generic Context Cognition non-authority, ontology completeness, and all dependent request constructors.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.
