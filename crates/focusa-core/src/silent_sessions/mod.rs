//! Canonical Spec 133 Silent Session domain contracts.
//!
//! This module contains typed, versioned facts only. Reducer transitions,
//! persistence, process supervision, adapters, and I/O live in later layers.

pub mod config;
pub mod identity;
pub mod persistence_records;
pub mod persistence_sqlite;
pub mod state_machine;
pub mod types;

pub use config::*;
pub use identity::*;
pub use persistence_records::*;
pub use persistence_sqlite::*;
pub use state_machine::*;
pub use types::*;

#[cfg(test)]
mod persistence_sqlite_test;
