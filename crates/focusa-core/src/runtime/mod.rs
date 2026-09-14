//! Runtime — daemon lifecycle, sessions, events, persistence.
//!
//! Source: G1-detail-03-runtime-daemon.md
//!
//! Process model:
//!   - Single daemon process
//!   - One Tokio runtime
//!   - State mutated via internal reducer (event-driven)
//!   - Concurrency: single owner task with mpsc command channel

pub mod backup;
pub mod backup_contracts;
pub mod backup_incremental;
mod backup_io;
pub mod backup_offhost;
pub mod backup_restore;
pub mod backup_retention;
#[cfg(test)]
mod backup_tests;
pub mod context_retrieval;
pub mod daemon;
pub mod event_bus;
pub mod event_retention;
pub mod events;
pub mod interview_strategy;
pub mod persistence;
pub mod persistence_actor;
pub mod persistence_sqlite;

#[cfg(test)]
mod persistence_sqlite_test;
