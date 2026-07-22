//! Canonical Spec 133 Silent Session domain contracts.
//!
//! This module contains typed, versioned facts only. Reducer transitions,
//! persistence, process supervision, adapters, and I/O live in later layers.

pub mod authorization;
pub mod authorization_persistence;
pub mod completion_artifacts;
pub mod config;
pub mod config_resolution;
pub mod config_revision;
pub mod event_protocol;
pub mod identity;
pub mod legacy_import;
pub mod persistence_records;
pub mod persistence_sqlite;
pub mod persistence_usage;
pub mod runner_security;
mod secure_fs;
pub mod state_machine;
pub mod stream_codec;
pub mod stream_recovery;
pub mod stream_rotation;
pub mod stream_storage;
pub mod types;

pub use authorization::*;
pub use authorization_persistence::*;
pub use completion_artifacts::*;
pub use config::*;
pub use config_resolution::*;
pub use config_revision::*;
pub use event_protocol::*;
pub use identity::*;
pub use legacy_import::*;
pub use persistence_records::*;
pub use persistence_sqlite::*;
pub use persistence_usage::*;
pub use runner_security::*;
pub use state_machine::*;
pub use stream_codec::*;
pub use stream_recovery::*;
pub use stream_rotation::*;
pub use stream_storage::*;
pub use types::*;

#[cfg(test)]
mod authorization_test;
#[cfg(test)]
mod config_resolution_test;
#[cfg(test)]
mod config_revision_test;
#[cfg(test)]
mod legacy_import_test;
#[cfg(test)]
mod persistence_sqlite_test;
#[cfg(test)]
mod runner_security_test;
#[cfg(test)]
mod stream_storage_test;
