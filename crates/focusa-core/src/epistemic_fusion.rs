//! Spec138 source fusion, missingness, independence, and contradiction authority.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightOrigin {
    DataEstimated,
    ModelEstimated,
    OperatorAssigned,
    PolicyAssigned,
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingnessKind {
    Unknown,
    Unavailable,
    NotApplicable,
    Redacted,
    Delayed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionSignal {
    pub signal_id: String,
    pub source_ref: String,
    pub upstream_dependency_refs: Vec<String>,
    pub owner_ref: String,
    pub acquisition_method_ref: String,
    pub redundancy_group_ref: Option<String>,
    pub manipulation_risk: f64,
    pub prompt_injection_risk: f64,
    pub source_revision: u64,
    pub first_available_at: DateTime<Utc>,
    pub value: Option<f64>,
    pub missingness: Option<MissingnessKind>,
    pub reliability: f64,
    pub assigned_weight: f64,
    pub weight_origin: WeightOrigin,
    pub correlation_cluster_ref: String,
    pub observed_at: DateTime<Utc>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionPolicy {
    pub policy_id: String,
    pub version: u64,
    pub minimum_independent_clusters: usize,
    pub contradiction_threshold: f64,
    pub maximum_source_risk: f64,
    pub normalize_weights: bool,
    pub preserve_contradictions: bool,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveContribution {
    pub signal_id: String,
    pub source_ref: String,
    pub correlation_cluster_ref: String,
    pub normalized_weight: f64,
    pub effective_weight: f64,
    pub value: f64,
    pub weighted_value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionResult {
    pub fusion_id: String,
    pub policy_ref: String,
    pub fused_value: f64,
    pub independent_cluster_count: usize,
    pub contributions: Vec<EffectiveContribution>,
    pub missing_signals: BTreeMap<String, MissingnessKind>,
    pub contradictory_signal_refs: Vec<String>,
    pub contradiction_preserved: bool,
    pub evidence_refs: Vec<String>,
    pub receipt_ref: String,
    pub fused_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FusionError {
    EmptySignals,
    DuplicateSignal,
    MissingIdentity,
    MissingEvidence,
    MissingReceipt,
    InvalidReliability,
    InvalidWeight,
    InvalidMissingness,
    InvalidPolicy,
    IndependenceMisclassified,
    SourceRiskExceeded,
    InsufficientIndependentSources,
    ZeroEffectiveWeight,
    ContradictionWouldBeHidden,
}

pub fn fuse_signals(
    fusion_id: impl Into<String>,
    signals: &[FusionSignal],
    policy: &FusionPolicy,
    receipt_ref: impl Into<String>,
    now: DateTime<Utc>,
) -> Result<FusionResult, FusionError> {
    if signals.is_empty() {
        return Err(FusionError::EmptySignals);
    }
    if policy.policy_id.trim().is_empty()
        || policy.version == 0
        || policy.minimum_independent_clusters == 0
        || !policy.contradiction_threshold.is_finite()
        || policy.contradiction_threshold < 0.0
        || !(0.0..=1.0).contains(&policy.maximum_source_risk)
        || policy.evidence_refs.is_empty()
    {
        return Err(FusionError::InvalidPolicy);
    }
    if policy.receipt_ref.trim().is_empty() {
        return Err(FusionError::MissingReceipt);
    }
    let receipt_ref = receipt_ref.into();
    if receipt_ref.trim().is_empty() {
        return Err(FusionError::MissingReceipt);
    }
    let mut ids = BTreeSet::new();
    let mut clusters = BTreeMap::<String, usize>::new();
    let mut missing_signals = BTreeMap::new();
    let mut present = Vec::new();
    let mut evidence_refs = policy.evidence_refs.clone();
    for signal in signals {
        if signal.signal_id.trim().is_empty()
            || signal.source_ref.trim().is_empty()
            || signal.owner_ref.trim().is_empty()
            || signal.acquisition_method_ref.trim().is_empty()
            || signal.source_revision == 0
            || signal.correlation_cluster_ref.trim().is_empty()
        {
            return Err(FusionError::MissingIdentity);
        }
        if !ids.insert(signal.signal_id.clone()) {
            return Err(FusionError::DuplicateSignal);
        }
        if signal.evidence_refs.is_empty() {
            return Err(FusionError::MissingEvidence);
        }
        if !(0.0..=1.0).contains(&signal.reliability)
            || !(0.0..=1.0).contains(&signal.manipulation_risk)
            || !(0.0..=1.0).contains(&signal.prompt_injection_risk)
        {
            return Err(FusionError::InvalidReliability);
        }
        if signal.manipulation_risk > policy.maximum_source_risk
            || signal.prompt_injection_risk > policy.maximum_source_risk
        {
            return Err(FusionError::SourceRiskExceeded);
        }
        if !signal.assigned_weight.is_finite() || signal.assigned_weight < 0.0 {
            return Err(FusionError::InvalidWeight);
        }
        match (signal.value, signal.missingness) {
            (Some(value), None) if value.is_finite() => {
                *clusters
                    .entry(signal.correlation_cluster_ref.clone())
                    .or_default() += 1;
                present.push(signal);
            }
            (None, Some(reason)) => {
                missing_signals.insert(signal.signal_id.clone(), reason);
            }
            _ => return Err(FusionError::InvalidMissingness),
        }
        evidence_refs.extend(signal.evidence_refs.clone());
    }
    for (index, left) in present.iter().enumerate() {
        for right in present.iter().skip(index + 1) {
            let shared_upstream = left
                .upstream_dependency_refs
                .iter()
                .any(|value| right.upstream_dependency_refs.contains(value));
            let dependent = shared_upstream
                || left.owner_ref == right.owner_ref
                || left
                    .redundancy_group_ref
                    .as_ref()
                    .is_some_and(|group| right.redundancy_group_ref.as_ref() == Some(group));
            if dependent && left.correlation_cluster_ref != right.correlation_cluster_ref {
                return Err(FusionError::IndependenceMisclassified);
            }
        }
    }
    if clusters.len() < policy.minimum_independent_clusters {
        return Err(FusionError::InsufficientIndependentSources);
    }
    let adjusted_weights = present
        .iter()
        .map(|signal| {
            signal.assigned_weight * signal.reliability
                / *clusters.get(&signal.correlation_cluster_ref).unwrap() as f64
        })
        .collect::<Vec<_>>();
    let total_weight = adjusted_weights.iter().sum::<f64>();
    if total_weight <= 0.0 {
        return Err(FusionError::ZeroEffectiveWeight);
    }
    let contributions = present
        .iter()
        .zip(adjusted_weights)
        .map(|(signal, adjusted)| {
            let normalized = if policy.normalize_weights {
                adjusted / total_weight
            } else {
                adjusted
            };
            let value = signal.value.unwrap();
            EffectiveContribution {
                signal_id: signal.signal_id.clone(),
                source_ref: signal.source_ref.clone(),
                correlation_cluster_ref: signal.correlation_cluster_ref.clone(),
                normalized_weight: normalized,
                effective_weight: adjusted,
                value,
                weighted_value: normalized * value,
            }
        })
        .collect::<Vec<_>>();
    let minimum = present
        .iter()
        .filter_map(|signal| signal.value)
        .fold(f64::INFINITY, f64::min);
    let maximum = present
        .iter()
        .filter_map(|signal| signal.value)
        .fold(f64::NEG_INFINITY, f64::max);
    let contradiction = maximum - minimum > policy.contradiction_threshold;
    if contradiction && !policy.preserve_contradictions {
        return Err(FusionError::ContradictionWouldBeHidden);
    }
    let contradictory_signal_refs = if contradiction {
        present
            .iter()
            .map(|signal| signal.signal_id.clone())
            .collect()
    } else {
        Vec::new()
    };
    let fused_value = contributions.iter().map(|row| row.weighted_value).sum();
    evidence_refs.sort();
    evidence_refs.dedup();
    Ok(FusionResult {
        fusion_id: fusion_id.into(),
        policy_ref: policy.policy_id.clone(),
        fused_value,
        independent_cluster_count: clusters.len(),
        contributions,
        missing_signals,
        contradictory_signal_refs,
        contradiction_preserved: contradiction,
        evidence_refs,
        receipt_ref,
        fused_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn signal(
        id: &str,
        value: Option<f64>,
        missingness: Option<MissingnessKind>,
        cluster: &str,
    ) -> FusionSignal {
        FusionSignal {
            signal_id: id.into(),
            source_ref: format!("source:{id}"),
            upstream_dependency_refs: vec![format!("upstream:{id}")],
            owner_ref: format!("owner:{id}"),
            acquisition_method_ref: "method:test".into(),
            redundancy_group_ref: None,
            manipulation_risk: 0.0,
            prompt_injection_risk: 0.0,
            source_revision: 1,
            first_available_at: Utc::now(),
            value,
            missingness,
            reliability: 1.0,
            assigned_weight: 1.0,
            weight_origin: WeightOrigin::PolicyAssigned,
            correlation_cluster_ref: cluster.into(),
            observed_at: Utc::now(),
            evidence_refs: vec![format!("evidence:{id}")],
        }
    }
    fn policy() -> FusionPolicy {
        FusionPolicy {
            policy_id: "fusion:v1".into(),
            version: 1,
            minimum_independent_clusters: 2,
            contradiction_threshold: 0.5,
            maximum_source_risk: 0.2,
            normalize_weights: true,
            preserve_contradictions: true,
            evidence_refs: vec!["evidence:policy".into()],
            receipt_ref: "receipt:policy".into(),
        }
    }
    #[test]
    fn missingness_is_not_zero_and_correlated_sources_are_discounted() {
        let result = fuse_signals(
            "fusion",
            &[
                signal("a", Some(0.8), None, "cluster-a"),
                signal("b", Some(0.6), None, "cluster-a"),
                signal("c", Some(0.7), None, "cluster-b"),
                signal(
                    "missing",
                    None,
                    Some(MissingnessKind::Unavailable),
                    "cluster-c",
                ),
            ],
            &policy(),
            "receipt:fusion",
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            result.missing_signals["missing"],
            MissingnessKind::Unavailable
        );
        assert_eq!(result.independent_cluster_count, 2);
        assert_eq!(result.contributions.len(), 3);
    }
    #[test]
    fn shared_upstream_cannot_masquerade_as_independent_confirmation() {
        let mut left = signal("a", Some(0.4), None, "cluster-a");
        let mut right = signal("b", Some(0.5), None, "cluster-b");
        left.upstream_dependency_refs = vec!["upstream:shared".into()];
        right.upstream_dependency_refs = vec!["upstream:shared".into()];
        assert_eq!(
            fuse_signals("fusion", &[left, right], &policy(), "receipt", Utc::now()),
            Err(FusionError::IndependenceMisclassified)
        );
        let mut risky = signal("risky", Some(0.6), None, "cluster-risk");
        risky.prompt_injection_risk = 0.9;
        assert_eq!(
            fuse_signals(
                "fusion",
                &[risky, signal("safe", Some(0.6), None, "cluster-safe")],
                &policy(),
                "receipt",
                Utc::now(),
            ),
            Err(FusionError::SourceRiskExceeded)
        );
    }

    #[test]
    fn contradiction_and_independence_fail_closed() {
        let mut hidden = policy();
        hidden.preserve_contradictions = false;
        assert_eq!(
            fuse_signals(
                "fusion",
                &[
                    signal("a", Some(0.1), None, "a"),
                    signal("b", Some(0.9), None, "b")
                ],
                &hidden,
                "receipt",
                Utc::now()
            ),
            Err(FusionError::ContradictionWouldBeHidden)
        );
        assert_eq!(
            fuse_signals(
                "fusion",
                &[
                    signal("a", Some(0.2), None, "same"),
                    signal("b", Some(0.3), None, "same")
                ],
                &policy(),
                "receipt",
                Utc::now()
            ),
            Err(FusionError::InsufficientIndependentSources)
        );
    }
}
