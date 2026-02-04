//! Focus Gate — pre-conscious salience filter.
//!
//! Source: 04-focus-gate.md, G1-detail-06-focus-gate.md
//!
//! INVARIANT: Never mutates Focus State or Focus Stack.
//! INVARIANT: Never triggers actions.
//! INVARIANT: Only surfaces candidates.
//! INVARIANT: All surfaced items are explainable.
//! INVARIANT: Decay and pressure are deterministic.

pub mod candidates;
pub mod focus_gate;
