//! Reference Store / ECS — externalized context store.
//!
//! Source: 07-reference-store.md, G1-detail-08-ecs.md
//!
//! INVARIANT: Artifacts are never implicitly injected.
//! INVARIANT: Artifacts are referenced by handles only.
//! INVARIANT: Artifacts are immutable once written.
//! INVARIANT: Rehydration is explicit and auditable.
//! INVARIANT: Storage is session-scoped by default.

pub mod artifact;
pub mod gc;
pub mod store;
