//! Runtime — daemon lifecycle, sessions, events, persistence.
//!
//! Source: G1-detail-03-runtime-daemon.md
//!
//! Process model:
//!   - Single daemon process
//!   - One Tokio runtime
//!   - State mutated via internal reducer (event-driven)
//!   - Concurrency: single owner task with mpsc command channel

pub mod daemon;
pub mod event_bus;
pub mod events;
pub mod persistence;
pub mod persistence_sqlite;

#[cfg(test)]
mod persistence_sqlite_test;
