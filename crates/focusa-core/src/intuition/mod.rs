//! Intuition Engine — async signal producer (subconscious).
//!
//! Source: 05-intuition-engine.md
//!
//! INVARIANT: Runs asynchronously only.
//! INVARIANT: Cannot block the hot path.
//! INVARIANT: Cannot mutate Focus State or Focus Stack.
//! INVARIANT: Emits signals, not commands.
//! INVARIANT: All signals are explainable.
//!
//! Forbidden: writing memory, altering focus, triggering actions,
//!            injecting prompt content.

pub mod aggregation;
pub mod engine;
pub mod signals;
