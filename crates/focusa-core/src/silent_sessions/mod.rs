//! Canonical Spec 133 Silent Session domain contracts.
//!
//! This module contains typed, versioned facts only. Reducer transitions,
//! persistence, process supervision, adapters, and I/O live in later layers.

pub mod config;
pub mod identity;
pub mod types;

pub use config::*;
pub use identity::*;
pub use types::*;
