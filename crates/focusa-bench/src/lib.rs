//! Focusa Benchmark Suite (Spec 113/114).
//!
//! Provides:
//! - model_matrix: pinned LLM provider/version/class/pricing metadata
//! - arms: no_focusa | passive_focusa | tool_only_focusa | full_focusa
//! - task: 150-task public/private benchmark suite
//! - metrics: Agent Power Index, Focusa Uplift Score, groundedness, etc.
//! - eval ledger API: append-only runs/events/complete/read/compare
//! - public-safe snapshots with redaction, hash chain, claim generation
//!
//! Design spec: docs/113-focusa-agent-performance-benchmark-spec.md
//! Design spec: docs/114-public-benchmark-flywheel-spec.md

pub mod arms;
pub mod ledger;
pub mod metrics;
pub mod model_matrix;
pub mod reports;
pub mod snapshot;
pub mod spec144;
pub mod task_suite;

#[cfg(test)]
mod tests;

// Re-exports
pub use arms::{Arm, ArmConfig};
pub use ledger::{EvalLedger, LedgerEntry, LedgerKind};
pub use metrics::{
    AgentPowerIndex, FocusaUpliftScore, GroundednessScore, HallucinationRate,
    OperatorBurdenReduction, PassAtN, TimeHorizon, ToolCallAccuracy,
};
pub use model_matrix::{ModelClass, ModelEntry, ModelMatrix};
pub use reports::{BenchmarkReport, MeasuredClaim, ReportArtifact};
pub use snapshot::{HashChain, PublicSnapshot, RedactionRule};
pub use spec144::{COMPARISON_COHORTS, EvaluationMetrics, PromotionThresholds};
pub use task_suite::{Task, TaskKind, TaskPool};

pub const BENCH_SCHEMA: &str = "focusa.bench.v1";
pub const PUBLIC_TASK_COUNT: usize = 75;
pub const PRIVATE_TASK_COUNT: usize = 75;
pub const TOTAL_TASK_COUNT: usize = PUBLIC_TASK_COUNT + PRIVATE_TASK_COUNT;

/// Library version: aligns with focusa workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
