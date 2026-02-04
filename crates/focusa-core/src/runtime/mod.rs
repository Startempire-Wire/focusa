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
pub mod events;
pub mod persistence;
