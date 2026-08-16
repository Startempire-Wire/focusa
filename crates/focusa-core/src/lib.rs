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
pub mod ascc;
pub mod autonomy;
pub mod awareness;
pub mod bloatgaurd;
pub mod bonjour;
pub mod cache;
pub mod claim_gate;
pub mod clt;
pub mod connector_auth;
pub mod connectors;
pub mod constitution;
pub mod dxux;
pub mod expression;
pub mod focus;
pub mod gate;
pub mod google_drive_connector;
pub mod intuition;
pub mod license;
pub mod license_developer_origin;
pub mod project_marker;
pub mod remote_workspace;
pub mod workstream_root;
pub mod compaction_policy;
pub mod memory;
pub mod permissions;
pub mod pre;
pub mod prediction;
pub mod provider_execution;
pub mod reducer;
pub mod reference;
pub mod replay;
pub mod rfm;
pub mod runtime;
pub mod scope_safety;
pub mod scoped_state;
pub mod silent_session;
pub mod silent_session_completion_events;
pub mod silent_session_authority;
pub mod silent_session_authorization;
pub mod silent_session_bootstrap;
pub mod silent_session_checkpoint_policy;
pub mod silent_session_completion;
pub mod silent_session_config;
pub mod silent_session_continuation;
pub mod silent_session_failure;
pub mod silent_session_integration;
pub mod silent_session_launch;
pub mod silent_session_notifications;
pub mod silent_session_protocol;
pub mod silent_session_receipts;
pub mod silent_session_reconstruction;
pub mod silent_session_recovery;
pub mod silent_session_reducer;
pub mod silent_session_resources;
pub mod silent_session_retry;
pub mod silent_session_scheduler;
pub mod silent_session_stream;
pub mod silent_session_wizard;
pub mod silent_session_workspace;
pub mod silent_session_writer;
pub mod silent_sessions;
pub mod skills;
pub mod software_domain;
pub mod background_job_store;
pub mod background_jobs;
pub mod callgraph;
pub mod completion_authority;
pub mod runtime_bundle;
pub mod runtime_constitution;
pub mod error_envelope;
pub mod infrastructure_inventory;
pub mod callgraph_envelope;
pub mod callgraph_export;
pub mod callgraph_store;
pub mod sync;
pub mod telemetry;
pub mod threads;
pub mod tool_result;
pub mod training;
pub mod types;
pub mod update;
pub mod utility_card;
pub mod uxp;
pub mod work_item;
pub mod workers;
pub mod working_subpath;
