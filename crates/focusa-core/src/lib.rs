//! Focusa Core — Cognitive runtime primitives and deterministic prompt assembly.
//!
//! This crate owns ALL cognition. CLI and API are thin facades.
//! No UI logic, no HTTP server wiring lives here.
//!
//! # Architecture
//!
//! - Single-writer reducer loop (event-driven)
//! - Deterministic state transitions
//! - Append-only event log
//! - Local filesystem persistence
//!
//! # Modules
//!
//! - `runtime` — Daemon lifecycle, sessions, events, persistence
//! - `focus` — Focus Stack (HEC), Focus Frames, Focus State
//! - `gate` — Focus Gate (salience filter), candidates
//! - `intuition` — Async signal producer (subconscious)
//! - `reference` — ECS / Reference Store (artifact offloading)
//! - `expression` — Prompt assembly engine
//! - `memory` — Semantic + procedural memory
//! - `workers` — Background cognition pipeline
//! - `adapters` — Harness adapters (proxy modes)
//! - `types` — Canonical shared types
//! - `reducer` — Core reducer (single writer)

pub mod adapters;
pub mod expression;
pub mod focus;
pub mod gate;
pub mod intuition;
pub mod memory;
pub mod reducer;
pub mod reference;
pub mod runtime;
pub mod types;
pub mod workers;
