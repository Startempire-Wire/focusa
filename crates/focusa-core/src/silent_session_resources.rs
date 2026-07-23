//! Resource admission, usage accounting, and pressure decisions for Silent Sessions.
//!
//! This module is pure policy. OS backends separately declare which limits they
//! can enforce; unsupported enforcement can never be represented as success.

use crate::silent_session::ResourceLimits;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const RESOURCE_ADMISSION_SCHEMA: &str = "focusa.resource_admission_decision.v1";
pub const RESOURCE_PRESSURE_SCHEMA: &str = "focusa.resource_pressure_decision.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetLevel {
    Turn,
    Run,
    Session,
    WorkItem,
    Project,
    User,
    ProviderModel,
    GlobalHost,
}

pub const ALL_BUDGET_LEVELS: [BudgetLevel; 8] = [
    BudgetLevel::Turn,
    BudgetLevel::Run,
    BudgetLevel::Session,
    BudgetLevel::WorkItem,
    BudgetLevel::Project,
    BudgetLevel::User,
    BudgetLevel::ProviderModel,
    BudgetLevel::GlobalHost,
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopedBudgetPolicy {
    pub max_tokens: Option<u64>,
    pub max_cost_usd: Option<f64>,
    pub max_turns: Option<u64>,
    pub max_output_bytes: Option<u64>,
    pub max_wall_clock_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceModeState {
    Normal,
    Constrained,
    Lowmem,
    Emergency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionQuotaSnapshot {
    pub active_global: u32,
    pub limit_global: u32,
    pub active_user: u32,
    pub limit_user: u32,
    pub active_project: u32,
    pub limit_project: u32,
    pub active_provider: u32,
    pub limit_provider: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionPrerequisites {
    pub writer_lease: bool,
    pub worktree_available: bool,
    pub runner_available: bool,
    pub model_entitled: bool,
    pub context_authority_verified: bool,
    pub workpoint_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostResourceSnapshot {
    pub available_cpu_percent: f64,
    pub available_memory_bytes: u64,
    pub available_disk_bytes: u64,
    pub available_stream_spool_bytes: u64,
    pub resource_mode: ResourceModeState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceDimension {
    Cpu,
    Memory,
    Pids,
    OpenFiles,
    Io,
    Disk,
    WallClock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementSupport {
    Native,
    Advisory,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEnforcementCapabilities {
    pub dimensions: BTreeMap<ResourceDimension, EnforcementSupport>,
}

impl ResourceEnforcementCapabilities {
    pub fn support(&self, dimension: ResourceDimension) -> EnforcementSupport {
        self.dimensions
            .get(&dimension)
            .copied()
            .unwrap_or(EnforcementSupport::Unsupported)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceAdmissionRequest {
    pub quotas: AdmissionQuotaSnapshot,
    pub prerequisites: AdmissionPrerequisites,
    pub host: HostResourceSnapshot,
    pub requested_limits: ResourceLimits,
    pub scoped_budgets: BTreeMap<BudgetLevel, ScopedBudgetPolicy>,
    pub required_native_enforcement: BTreeSet<ResourceDimension>,
    pub enforcement: ResourceEnforcementCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionDenial {
    GlobalQuota,
    UserQuota,
    ProjectQuota,
    ProviderQuota,
    CpuUnavailable,
    MemoryUnavailable,
    DiskUnavailable,
    StreamSpoolUnavailable,
    WriterLeaseMissing,
    WorktreeUnavailable,
    RunnerUnavailable,
    ModelEntitlementMissing,
    ContextAuthorityMissing,
    WorkpointNotReady,
    EmergencyResourceMode,
    IncompleteBudgetLevels,
    InvalidBudget,
    UnsupportedNativeEnforcement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdmissionDecision {
    pub schema: String,
    pub admitted: bool,
    pub degraded: bool,
    pub denials: Vec<AdmissionDenial>,
}

impl ResourceAdmissionRequest {
    pub fn evaluate(&self) -> ResourceAdmissionDecision {
        let mut denials = Vec::new();
        if self.quotas.limit_global == 0 || self.quotas.active_global >= self.quotas.limit_global {
            denials.push(AdmissionDenial::GlobalQuota);
        }
        if self.quotas.limit_user == 0 || self.quotas.active_user >= self.quotas.limit_user {
            denials.push(AdmissionDenial::UserQuota);
        }
        if self.quotas.limit_project == 0 || self.quotas.active_project >= self.quotas.limit_project
        {
            denials.push(AdmissionDenial::ProjectQuota);
        }
        if self.quotas.limit_provider == 0
            || self.quotas.active_provider >= self.quotas.limit_provider
        {
            denials.push(AdmissionDenial::ProviderQuota);
        }
        if !self.prerequisites.writer_lease {
            denials.push(AdmissionDenial::WriterLeaseMissing);
        }
        if !self.prerequisites.worktree_available {
            denials.push(AdmissionDenial::WorktreeUnavailable);
        }
        if !self.prerequisites.runner_available {
            denials.push(AdmissionDenial::RunnerUnavailable);
        }
        if !self.prerequisites.model_entitled {
            denials.push(AdmissionDenial::ModelEntitlementMissing);
        }
        if !self.prerequisites.context_authority_verified {
            denials.push(AdmissionDenial::ContextAuthorityMissing);
        }
        if !self.prerequisites.workpoint_ready {
            denials.push(AdmissionDenial::WorkpointNotReady);
        }
        if self.host.resource_mode == ResourceModeState::Emergency {
            denials.push(AdmissionDenial::EmergencyResourceMode);
        }
        if !self.host.available_cpu_percent.is_finite() || self.host.available_cpu_percent <= 0.0 {
            denials.push(AdmissionDenial::CpuUnavailable);
        }
        if self
            .requested_limits
            .max_memory_bytes
            .is_some_and(|requested| requested > self.host.available_memory_bytes)
        {
            denials.push(AdmissionDenial::MemoryUnavailable);
        }
        if self
            .requested_limits
            .max_disk_bytes
            .is_some_and(|requested| requested > self.host.available_disk_bytes)
        {
            denials.push(AdmissionDenial::DiskUnavailable);
        }
        if self
            .requested_limits
            .max_output_bytes
            .is_some_and(|requested| requested > self.host.available_stream_spool_bytes)
        {
            denials.push(AdmissionDenial::StreamSpoolUnavailable);
        }
        if self.scoped_budgets.len() != ALL_BUDGET_LEVELS.len()
            || ALL_BUDGET_LEVELS
                .iter()
                .any(|level| !self.scoped_budgets.contains_key(level))
        {
            denials.push(AdmissionDenial::IncompleteBudgetLevels);
        } else if self.scoped_budgets.values().any(invalid_budget) {
            denials.push(AdmissionDenial::InvalidBudget);
        }
        if self
            .required_native_enforcement
            .iter()
            .any(|dimension| self.enforcement.support(*dimension) != EnforcementSupport::Native)
        {
            denials.push(AdmissionDenial::UnsupportedNativeEnforcement);
        }
        denials.sort_by_key(|denial| *denial as u8);
        denials.dedup();
        ResourceAdmissionDecision {
            schema: RESOURCE_ADMISSION_SCHEMA.into(),
            admitted: denials.is_empty(),
            degraded: denials.is_empty()
                && matches!(
                    self.host.resource_mode,
                    ResourceModeState::Constrained | ResourceModeState::Lowmem
                ),
            denials,
        }
    }
}

fn invalid_budget(budget: &ScopedBudgetPolicy) -> bool {
    budget.max_tokens == Some(0)
        || budget.max_turns == Some(0)
        || budget.max_output_bytes == Some(0)
        || budget.max_wall_clock_seconds == Some(0)
        || budget
            .max_cost_usd
            .is_some_and(|value| !value.is_finite() || value <= 0.0)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    pub cpu_percent: Option<f64>,
    pub memory_bytes: Option<u64>,
    pub pids: Option<u32>,
    pub open_files: Option<u32>,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
    pub disk_bytes: Option<u64>,
    pub wall_clock_seconds: u64,
    pub output_bytes: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub estimated_cost_usd: Option<f64>,
    pub provider_reported_cost_usd: Option<f64>,
    pub subscription_usage: Option<f64>,
    pub context_tokens: Option<u64>,
    pub context_window: Option<u64>,
    pub retry_waste_tokens: u64,
    pub turns: u64,
}

impl ResourceUsageSnapshot {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens
            .saturating_add(self.output_tokens)
            .saturating_add(self.cache_read_tokens.unwrap_or(0))
            .saturating_add(self.cache_write_tokens.unwrap_or(0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardLimitAction {
    CheckpointAndPause,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePressurePolicy {
    pub warning_basis_points: u16,
    pub hard_limit_action: HardLimitAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePressurePosture {
    Healthy,
    Warning,
    CheckpointAndPause,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePressureReason {
    Cpu,
    Memory,
    Pids,
    Disk,
    WallClock,
    Output,
    Tokens,
    Cost,
    Turns,
    ContextWindow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePressureDecision {
    pub schema: String,
    pub posture: ResourcePressurePosture,
    pub warning_reasons: Vec<ResourcePressureReason>,
    pub hard_reasons: Vec<ResourcePressureReason>,
    pub checkpoint_required: bool,
    pub pause_required: bool,
    pub cancel_required: bool,
    pub full_usage_truth_preserved: bool,
}

pub fn evaluate_resource_pressure(
    limits: &ResourceLimits,
    usage: &ResourceUsageSnapshot,
    policy: ResourcePressurePolicy,
) -> ResourcePressureDecision {
    let threshold = policy.warning_basis_points.clamp(1, 9_999);
    let mut warning_reasons = Vec::new();
    let mut hard_reasons = Vec::new();

    classify_f64(
        ResourcePressureReason::Cpu,
        usage.cpu_percent,
        limits.max_cpu_percent,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_u64(
        ResourcePressureReason::Memory,
        usage.memory_bytes,
        limits.max_memory_bytes,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_u64(
        ResourcePressureReason::Pids,
        usage.pids.map(u64::from),
        limits.max_pids.map(u64::from),
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_u64(
        ResourcePressureReason::Disk,
        usage.disk_bytes,
        limits.max_disk_bytes,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_u64(
        ResourcePressureReason::WallClock,
        Some(usage.wall_clock_seconds),
        limits.max_wall_clock_seconds,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_u64(
        ResourcePressureReason::Output,
        Some(usage.output_bytes),
        limits.max_output_bytes,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_u64(
        ResourcePressureReason::Tokens,
        Some(usage.total_tokens()),
        limits.max_tokens,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_f64(
        ResourcePressureReason::Cost,
        usage
            .provider_reported_cost_usd
            .or(usage.estimated_cost_usd),
        limits.max_cost_usd,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    classify_u64(
        ResourcePressureReason::Turns,
        Some(usage.turns),
        limits.max_turns,
        threshold,
        &mut warning_reasons,
        &mut hard_reasons,
    );
    if let (Some(tokens), Some(window)) = (usage.context_tokens, usage.context_window) {
        classify_u64(
            ResourcePressureReason::ContextWindow,
            Some(tokens),
            Some(window),
            threshold,
            &mut warning_reasons,
            &mut hard_reasons,
        );
    }
    warning_reasons.sort();
    warning_reasons.dedup();
    hard_reasons.sort();
    hard_reasons.dedup();

    let posture = if hard_reasons.is_empty() {
        if warning_reasons.is_empty() {
            ResourcePressurePosture::Healthy
        } else {
            ResourcePressurePosture::Warning
        }
    } else {
        match policy.hard_limit_action {
            HardLimitAction::CheckpointAndPause => ResourcePressurePosture::CheckpointAndPause,
            HardLimitAction::Cancel => ResourcePressurePosture::Cancel,
        }
    };
    ResourcePressureDecision {
        schema: RESOURCE_PRESSURE_SCHEMA.into(),
        posture,
        warning_reasons,
        hard_reasons,
        checkpoint_required: posture == ResourcePressurePosture::CheckpointAndPause,
        pause_required: posture == ResourcePressurePosture::CheckpointAndPause,
        cancel_required: posture == ResourcePressurePosture::Cancel,
        full_usage_truth_preserved: true,
    }
}

fn classify_u64(
    reason: ResourcePressureReason,
    actual: Option<u64>,
    limit: Option<u64>,
    threshold: u16,
    warnings: &mut Vec<ResourcePressureReason>,
    hard: &mut Vec<ResourcePressureReason>,
) {
    let (Some(actual), Some(limit)) = (actual, limit) else {
        return;
    };
    if actual >= limit {
        hard.push(reason);
    } else if u128::from(actual) * 10_000 >= u128::from(limit) * u128::from(threshold) {
        warnings.push(reason);
    }
}

fn classify_f64(
    reason: ResourcePressureReason,
    actual: Option<f64>,
    limit: Option<f64>,
    threshold: u16,
    warnings: &mut Vec<ResourcePressureReason>,
    hard: &mut Vec<ResourcePressureReason>,
) {
    let (Some(actual), Some(limit)) = (actual, limit) else {
        return;
    };
    if !actual.is_finite() || !limit.is_finite() || limit <= 0.0 || actual >= limit {
        hard.push(reason);
    } else if actual / limit >= f64::from(threshold) / 10_000.0 {
        warnings.push(reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budgets() -> BTreeMap<BudgetLevel, ScopedBudgetPolicy> {
        ALL_BUDGET_LEVELS
            .into_iter()
            .map(|level| {
                (
                    level,
                    ScopedBudgetPolicy {
                        max_tokens: Some(10_000),
                        max_cost_usd: Some(10.0),
                        max_turns: Some(100),
                        max_output_bytes: Some(1_000_000),
                        max_wall_clock_seconds: Some(3_600),
                    },
                )
            })
            .collect()
    }

    fn limits() -> ResourceLimits {
        ResourceLimits {
            priority: 0,
            max_wall_clock_seconds: Some(100),
            max_cpu_percent: Some(100.0),
            max_memory_bytes: Some(1_000),
            max_pids: Some(10),
            max_disk_bytes: Some(10_000),
            max_output_bytes: Some(5_000),
            max_tokens: Some(1_000),
            max_cost_usd: Some(5.0),
            max_turns: Some(10),
        }
    }

    fn admission() -> ResourceAdmissionRequest {
        ResourceAdmissionRequest {
            quotas: AdmissionQuotaSnapshot {
                active_global: 1,
                limit_global: 10,
                active_user: 1,
                limit_user: 4,
                active_project: 1,
                limit_project: 3,
                active_provider: 1,
                limit_provider: 5,
            },
            prerequisites: AdmissionPrerequisites {
                writer_lease: true,
                worktree_available: true,
                runner_available: true,
                model_entitled: true,
                context_authority_verified: true,
                workpoint_ready: true,
            },
            host: HostResourceSnapshot {
                available_cpu_percent: 75.0,
                available_memory_bytes: 2_000,
                available_disk_bytes: 20_000,
                available_stream_spool_bytes: 10_000,
                resource_mode: ResourceModeState::Normal,
            },
            requested_limits: limits(),
            scoped_budgets: budgets(),
            required_native_enforcement: BTreeSet::new(),
            enforcement: ResourceEnforcementCapabilities {
                dimensions: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn admission_requires_all_scopes_prerequisites_capacity_and_capability_truth() {
        assert!(admission().evaluate().admitted);
        let mut blocked = admission();
        blocked.quotas.active_project = blocked.quotas.limit_project;
        blocked.prerequisites.writer_lease = false;
        blocked.host.resource_mode = ResourceModeState::Emergency;
        blocked
            .required_native_enforcement
            .insert(ResourceDimension::Memory);
        let decision = blocked.evaluate();
        assert!(!decision.admitted);
        assert!(decision.denials.contains(&AdmissionDenial::ProjectQuota));
        assert!(
            decision
                .denials
                .contains(&AdmissionDenial::WriterLeaseMissing)
        );
        assert!(
            decision
                .denials
                .contains(&AdmissionDenial::EmergencyResourceMode)
        );
        assert!(
            decision
                .denials
                .contains(&AdmissionDenial::UnsupportedNativeEnforcement)
        );

        let mut incomplete = admission();
        incomplete.scoped_budgets.remove(&BudgetLevel::GlobalHost);
        assert!(
            incomplete
                .evaluate()
                .denials
                .contains(&AdmissionDenial::IncompleteBudgetLevels)
        );
    }

    #[test]
    fn usage_warning_and_hard_limit_actions_preserve_complete_truth() {
        let mut usage = ResourceUsageSnapshot {
            cpu_percent: Some(50.0),
            memory_bytes: Some(850),
            pids: Some(2),
            open_files: Some(10),
            io_read_bytes: Some(100),
            io_write_bytes: Some(200),
            disk_bytes: Some(1_000),
            wall_clock_seconds: 20,
            output_bytes: 1_000,
            input_tokens: 300,
            output_tokens: 100,
            cache_read_tokens: Some(450),
            cache_write_tokens: Some(0),
            estimated_cost_usd: Some(1.0),
            provider_reported_cost_usd: None,
            subscription_usage: None,
            context_tokens: Some(70),
            context_window: Some(100),
            retry_waste_tokens: 25,
            turns: 2,
        };
        let policy = ResourcePressurePolicy {
            warning_basis_points: 8_000,
            hard_limit_action: HardLimitAction::CheckpointAndPause,
        };
        let warning = evaluate_resource_pressure(&limits(), &usage, policy);
        assert_eq!(warning.posture, ResourcePressurePosture::Warning);
        assert!(
            warning
                .warning_reasons
                .contains(&ResourcePressureReason::Memory)
        );
        assert!(
            warning
                .warning_reasons
                .contains(&ResourcePressureReason::Tokens)
        );
        assert!(warning.full_usage_truth_preserved);

        usage.output_bytes = 5_000;
        let hard = evaluate_resource_pressure(&limits(), &usage, policy);
        assert_eq!(hard.posture, ResourcePressurePosture::CheckpointAndPause);
        assert!(hard.checkpoint_required && hard.pause_required && !hard.cancel_required);

        let cancelled = evaluate_resource_pressure(
            &limits(),
            &usage,
            ResourcePressurePolicy {
                hard_limit_action: HardLimitAction::Cancel,
                ..policy
            },
        );
        assert_eq!(cancelled.posture, ResourcePressurePosture::Cancel);
        assert!(cancelled.cancel_required);
    }
}
