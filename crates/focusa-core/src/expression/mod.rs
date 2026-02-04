//! Expression Engine — deterministic prompt assembly.
//!
//! Source: 08-expression-engine.md, G1-detail-11-prompt-assembly.md
//!
//! INVARIANT: Deterministic output.
//! INVARIANT: Explicit structure.
//! INVARIANT: Bounded token usage.
//! INVARIANT: No silent truncation.
//! INVARIANT: No reasoning or planning.
//!
//! Think "compiler," not "LLM magic."

pub mod budget;
pub mod engine;
pub mod serializer;
