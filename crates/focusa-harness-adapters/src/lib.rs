//! Harness-neutral, versioned adapter boundary for daemon-native Silent
//! Sessions.
//!
//! This crate translates harness protocols. It does not own canonical session
//! state, process lifecycle, authorization, or completion authority.

pub mod contract;
pub mod fake;
pub mod generic;
pub mod pi_rpc;

pub use contract::*;
pub use fake::*;
pub use generic::*;
pub use pi_rpc::*;
