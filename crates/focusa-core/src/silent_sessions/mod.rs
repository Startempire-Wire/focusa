//! Canonical Spec 133 Silent Session domain contracts.
//!
//! This module contains typed, versioned facts only. Reducer transitions,
//! persistence, process supervision, adapters, and I/O live in later layers.

pub mod authorization;
pub mod authorization_persistence;
pub mod capability_catalog;
pub mod cognitive_governance;
pub mod completion_artifacts;
pub mod concurrency_governance;
pub mod config;
pub mod config_resolution;
pub mod config_revision;
pub mod event_protocol;
pub mod failure_envelope;
pub mod harness_adapter;
pub mod identity;
pub mod launch_manifest;
pub mod legacy_import;
pub mod model_safety;
pub mod operator_experience;
pub mod persistence_records;
pub mod persistence_sqlite;
pub mod persistence_usage;
pub mod pi_rpc_adapter;
pub mod platform_backends;
pub mod process_supervision;
pub mod recovery_policy;
pub mod resource_admission;
pub mod retention;
pub mod runner_client;
pub mod runner_protocol;
pub mod runner_security;
pub mod runtime_control;
mod secure_fs;
pub mod state_machine;
pub mod stream_codec;
pub mod stream_recovery;
pub mod stream_rotation;
pub mod stream_storage;
pub mod types;

pub use authorization::*;
pub use authorization_persistence::*;
pub use capability_catalog::*;
pub use cognitive_governance::*;
pub use completion_artifacts::*;
pub use concurrency_governance::*;
pub use config::*;
pub use config_resolution::*;
pub use config_revision::*;
pub use event_protocol::*;
pub use failure_envelope::*;
pub use harness_adapter::*;
pub use identity::*;
pub use launch_manifest::*;
pub use legacy_import::*;
pub use model_safety::*;
pub use operator_experience::*;
pub use persistence_records::*;
pub use persistence_sqlite::*;
pub use persistence_usage::*;
pub use pi_rpc_adapter::*;
pub use platform_backends::*;
pub use process_supervision::*;
pub use recovery_policy::*;
pub use resource_admission::*;
pub use retention::*;
pub use runner_client::*;
pub use runner_protocol::*;
pub use runner_security::*;
pub use runtime_control::*;
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
mod harness_adapter_test;
#[cfg(test)]
mod launch_manifest_test;
#[cfg(test)]
mod legacy_import_test;
#[cfg(test)]
mod persistence_sqlite_test;
#[cfg(test)]
mod runner_protocol_test;
#[cfg(test)]
mod runner_security_test;
#[cfg(test)]
mod stream_storage_test;
