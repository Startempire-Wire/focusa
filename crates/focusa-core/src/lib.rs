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
pub mod agent_runtime_constitution;
pub mod agent_runtime_constitution_authority;
#[cfg(test)]
mod agent_runtime_constitution_authority_test;
pub mod agent_runtime_constitution_compiler;
#[cfg(test)]
mod agent_runtime_constitution_compiler_test;
pub mod agent_runtime_constitution_enforcement;
#[cfg(test)]
mod agent_runtime_constitution_enforcement_test;
pub mod agent_runtime_constitution_lifecycle;
#[cfg(test)]
mod agent_runtime_constitution_lifecycle_test;
pub mod agent_runtime_constitution_migration;
#[cfg(test)]
mod agent_runtime_constitution_migration_test;
pub mod agent_runtime_constitution_orchestrator;
#[cfg(test)]
mod agent_runtime_constitution_orchestrator_test;
pub mod agent_runtime_constitution_store;
#[cfg(test)]
mod agent_runtime_constitution_store_test;
#[cfg(test)]
mod agent_runtime_constitution_test;
pub mod agent_runtime_instruction_integrity;
#[cfg(test)]
mod agent_runtime_instruction_integrity_scenario_test;
pub mod ascc;
pub mod autonomy;
pub mod awareness;
pub mod bloatgaurd;
pub mod bonjour;
pub mod cache;
pub mod claim_gate;
pub mod clt;
pub mod compaction_policy;
pub mod connector_auth;
pub mod connectors;
pub mod constitution;
pub mod convergence_platform;
pub mod convergence_transaction;
pub mod daemon_auth;
pub mod daemon_dispatch;
pub mod daemon_identity;
pub mod daemon_multiplex;
pub mod dxux;
pub mod epistemic_conformance;
pub mod epistemic_fusion;
pub mod epistemic_memory_lifecycle;
pub mod epistemic_primitives;
pub mod epistemic_security;
pub mod expression;
pub mod focus;
pub mod gate;
pub mod google_drive_connector;
pub mod guarded_mutation;
pub mod install_lifecycle;
pub mod installation_convergence;
pub mod intuition;
pub mod entitlement_execution_guard;
pub mod license;
pub mod limited_project;
pub mod memory;
pub mod metacognitive_learning;
pub mod outcome_resolution;
pub mod permissions;
pub mod pre;
pub mod prediction;
pub mod prediction_advanced;
pub mod prediction_authority;
pub mod prediction_authority_ledger;
pub mod prediction_authority_storage;
pub mod prediction_authority_validation;
#[cfg(test)]
#[path = "prediction_authority_tests.rs"]
mod prediction_authority_tests;
#[cfg(test)]
mod prediction_authority_runtime_tests;
pub mod prediction_calibration;
pub mod prediction_migration;
pub mod prediction_profiles;
pub mod prediction_scoring;
pub mod prediction_scoring_algorithms;
pub mod spec138_operations;
pub mod provider_execution;
pub mod reducer;
pub mod reference;
pub mod release_adapters;
pub mod runtime_entrypoint_classification;
pub mod release_calibration;
pub mod release_cycle;
pub mod release_intelligence;
pub mod release_ledger;
pub mod release_orchestrator;
pub mod release_planner;
pub mod release_protocol;
pub mod replay;
pub mod rfm;
pub mod runtime;
pub mod scope_safety;
pub mod scoped_state;
pub mod semantic_integrity;
pub mod semantic_migration;
#[cfg(test)]
mod semantic_migration_tests;
pub mod semantic_pair;
#[cfg(test)]
mod semantic_pair_tests;
pub mod semantic_performance;
pub mod semantic_reflex;
pub mod semantic_registry;
pub mod semantic_replay;
#[cfg(test)]
mod semantic_replay_tests;
pub mod semantic_security;
pub mod semantic_settlement;
pub mod semantic_verification;
pub mod semantic_vertical;
pub mod silent_session;
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
pub mod sync;
pub mod telemetry;
pub mod temporal;
pub mod temporal_authority;
pub mod temporal_calibration_loop;
pub mod temporal_claims;
pub mod temporal_clock;
pub mod temporal_closure;
pub mod temporal_conformance;
pub mod temporal_deadline;
pub mod temporal_forecast;
pub mod temporal_forecast_evaluation;
pub mod temporal_foundation;
#[cfg(test)]
mod temporal_full_tests;
pub mod temporal_high_consequence;
pub mod temporal_integrity;
pub mod temporal_ledger;
pub mod temporal_operations;
pub mod temporal_platform;
pub mod temporal_progress;
pub mod temporal_release_gate;
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
pub mod acceptance_atoms;
pub mod adapter_registry;
pub mod background_job_store;
pub mod background_jobs;
pub mod callgraph;
pub mod callgraph_envelope;
pub mod callgraph_export;
pub mod callgraph_store;
pub mod capability_truth;
pub mod completion_authority;
pub mod credential_authority;
pub mod direction_ledger;
pub mod direction_operations;
pub mod error_envelope;
pub mod infrastructure_inventory;
pub mod license_developer_origin;
pub mod procedure_compiler;
pub mod project_marker;
pub mod remote_workspace;
pub mod runtime_bundle;
pub mod runtime_constitution;
pub mod session_fanout;
pub mod silent_session_completion_events;
pub mod workset_context;
pub mod workset_freshness;
pub mod workset_ledger;
pub mod workset_providers;
pub mod workset_store;
pub mod workset_transitions;
pub mod workstream_root;

pub use entitlement_execution_guard::{
    evaluate_entitlement_execution,
    EntitlementExecutionContext,
    EntitlementExecutionDecision,
    EntitlementExecutionFailure,
    EntitlementExecutionPolicy,
};
