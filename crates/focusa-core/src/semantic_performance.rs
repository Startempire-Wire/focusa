//! Spec 144 §27 bounded semantic execution and pressure-safe planning.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticExecutionMode {
    Cached,
    AffectedNeighborhood,
    WholeWorldAsync,
    DeferredPreservingAcceptedWork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticPressureMode {
    Normal,
    Constrained,
    LowMemory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssuranceStrength {
    Advisory,
    Standard,
    Independent,
    HighConsequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticExecutionRequest {
    pub artifact_hash: String,
    pub profile_hash: String,
    pub changed_node_refs: BTreeSet<String>,
    pub affected_node_refs: BTreeSet<String>,
    pub estimated_nodes: u64,
    pub estimated_memory_bytes: u64,
    pub whole_world_reasoning_required: bool,
    pub required_strength: AssuranceStrength,
    pub accepted_work_refs: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticPerformancePolicy {
    pub max_sync_nodes: u64,
    pub max_sync_memory_bytes: u64,
    pub max_affected_nodes: u64,
    pub allow_advisory_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticExecutionPlan {
    pub mode: SemanticExecutionMode,
    pub cache_key: String,
    pub validation_node_refs: BTreeSet<String>,
    pub preserved_accepted_work_refs: BTreeSet<String>,
    pub achieved_strength: AssuranceStrength,
    pub result_limit_required: bool,
    pub cancellation_required: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SemanticPerformanceError {
    #[error("semantic artifact/profile hash is missing")]
    MissingCacheIdentity,
    #[error("affected semantic neighborhood exceeds its bound")]
    AffectedNeighborhoodTooLarge,
    #[error("resource pressure may not weaken mandatory assurance")]
    AssuranceDowngradeForbidden,
}

pub fn plan_semantic_execution(
    policy: &SemanticPerformancePolicy,
    request: &SemanticExecutionRequest,
    pressure: SemanticPressureMode,
    cached_keys: &BTreeSet<String>,
) -> Result<SemanticExecutionPlan, SemanticPerformanceError> {
    if request.artifact_hash.is_empty() || request.profile_hash.is_empty() {
        return Err(SemanticPerformanceError::MissingCacheIdentity);
    }
    let cache_key = format!("{}:{}", request.artifact_hash, request.profile_hash);
    if cached_keys.contains(&cache_key) {
        return Ok(SemanticExecutionPlan {
            mode: SemanticExecutionMode::Cached,
            cache_key,
            validation_node_refs: BTreeSet::new(),
            preserved_accepted_work_refs: request.accepted_work_refs.clone(),
            achieved_strength: request.required_strength.clone(),
            result_limit_required: true,
            cancellation_required: false,
        });
    }
    if request.affected_node_refs.len() as u64 > policy.max_affected_nodes {
        return Err(SemanticPerformanceError::AffectedNeighborhoodTooLarge);
    }
    let over_sync_budget = request.estimated_nodes > policy.max_sync_nodes
        || request.estimated_memory_bytes > policy.max_sync_memory_bytes;
    let pressure_requires_defer = pressure != SemanticPressureMode::Normal && over_sync_budget;
    if pressure_requires_defer
        && request.required_strength != AssuranceStrength::Advisory
        && policy.allow_advisory_fallback
    {
        return Err(SemanticPerformanceError::AssuranceDowngradeForbidden);
    }
    let mode = if pressure_requires_defer {
        SemanticExecutionMode::DeferredPreservingAcceptedWork
    } else if request.whole_world_reasoning_required || over_sync_budget {
        SemanticExecutionMode::WholeWorldAsync
    } else {
        SemanticExecutionMode::AffectedNeighborhood
    };
    let validation_node_refs = if mode == SemanticExecutionMode::AffectedNeighborhood {
        request
            .changed_node_refs
            .union(&request.affected_node_refs)
            .cloned()
            .collect()
    } else {
        BTreeSet::new()
    };
    Ok(SemanticExecutionPlan {
        mode,
        cache_key,
        validation_node_refs,
        preserved_accepted_work_refs: request.accepted_work_refs.clone(),
        achieved_strength: request.required_strength.clone(),
        result_limit_required: true,
        cancellation_required: true,
    })
}

#[cfg(test)]
#[path = "semantic_performance_tests.rs"]
mod tests;
