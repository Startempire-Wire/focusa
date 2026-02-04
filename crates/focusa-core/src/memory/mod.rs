//! Memory — semantic + procedural.
//!
//! Source: G1-09-memory.md
//!
//! INVARIANT: Memory is opt-in.
//! INVARIANT: Memory writes require explicit user command or confirmed promotion.
//! INVARIANT: No automatic personality drift.
//! INVARIANT: No silent preference learning.
//! INVARIANT: No speculative inference.

pub mod procedural;
pub mod semantic;
