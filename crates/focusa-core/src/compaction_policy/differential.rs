use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DifferentialCycle {
    pub cycle_id: String,
    pub operator_turn_ids_in: Vec<String>,
    pub operator_turn_ids_out: Vec<String>,
    pub model_turn_ids: Vec<String>,
    pub expected_project_hash: String,
    pub actual_project_hash: String,
    pub opaque_state_digest_before: Option<String>,
    pub opaque_state_digest_after: Option<String>,
    pub recovery_handles_before: Vec<String>,
    pub recovery_handles_after: Vec<String>,
    pub workpoint_revision_delta: i64,
    pub trajectory_current: bool,
    pub active_blocker_truthful: bool,
    pub correct_next_action: bool,
    pub task_success: bool,
    pub tokens_before: u64,
    pub tokens_after: u64,
    pub billed_tokens: u64,
    pub latency_ms: u64,
    pub cache_behavior: String,
    pub prepare_rpc_count: u8,
    pub verify_rpc_count: u8,
    pub observation_batch_count: u8,
    pub extra_summarizer_calls: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DifferentialRun {
    pub label: String,
    pub cycles: Vec<DifferentialCycle>,
    pub restart_recovered: bool,
    pub transport_fallback_recovered: bool,
    pub rollback_drill_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DifferentialAcceptanceInput {
    pub schema: String,
    pub runtime_segment: String,
    pub provider_strategy: String,
    pub baseline_without_focusa: DifferentialRun,
    pub legacy_focusa: DifferentialRun,
    pub adaptive_focusa: DifferentialRun,
    pub noninferiority_epsilon: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DifferentialAcceptanceReceipt {
    pub schema: String,
    pub status: String,
    pub runtime_segment: String,
    pub provider_strategy: String,
    pub cycles_per_run: usize,
    pub findings: Vec<String>,
    pub baseline_task_success_rate: f64,
    pub legacy_task_success_rate: f64,
    pub adaptive_task_success_rate: f64,
    pub adaptive_next_action_rate: f64,
    pub adaptive_context_release_ratio: f64,
    pub adaptive_billed_tokens: u64,
    pub adaptive_latency_ms: u64,
    pub evidence_digest: String,
}

fn rate(cycles: &[DifferentialCycle], predicate: impl Fn(&DifferentialCycle) -> bool) -> f64 {
    if cycles.is_empty() {
        return 0.0;
    }
    cycles.iter().filter(|cycle| predicate(cycle)).count() as f64 / cycles.len() as f64
}

fn hard_findings(run: &DifferentialRun) -> Vec<String> {
    let mut findings = Vec::new();
    for cycle in &run.cycles {
        let prefix = format!("{}:{}", run.label, cycle.cycle_id);
        if cycle.operator_turn_ids_in != cycle.operator_turn_ids_out {
            findings.push(format!("{prefix}:operator_turn_loss_or_reorder"));
        }
        let unique_model_turns: BTreeSet<_> = cycle.model_turn_ids.iter().collect();
        if unique_model_turns.len() != cycle.model_turn_ids.len() {
            findings.push(format!("{prefix}:duplicate_model_turn"));
        }
        if cycle.expected_project_hash != cycle.actual_project_hash {
            findings.push(format!("{prefix}:foreign_scope"));
        }
        if cycle.opaque_state_digest_before != cycle.opaque_state_digest_after {
            findings.push(format!("{prefix}:opaque_state_loss"));
        }
        if cycle.recovery_handles_before != cycle.recovery_handles_after {
            findings.push(format!("{prefix}:recovery_handle_loss"));
        }
        if cycle.workpoint_revision_delta < 0
            || !cycle.trajectory_current
            || !cycle.active_blocker_truthful
        {
            findings.push(format!("{prefix}:authority_regression"));
        }
        if cycle.tokens_after >= cycle.tokens_before {
            findings.push(format!("{prefix}:ineffective_context_release"));
        }
        if cycle.cache_behavior.trim().is_empty() || cycle.cache_behavior == "unclassified" {
            findings.push(format!("{prefix}:cache_unclassified"));
        }
        if cycle.prepare_rpc_count > 1
            || cycle.verify_rpc_count > 1
            || cycle.observation_batch_count > 1
            || cycle.extra_summarizer_calls > 0
        {
            findings.push(format!("{prefix}:performance_budget_exceeded"));
        }
    }
    if !run.restart_recovered {
        findings.push(format!("{}:restart_recovery_failed", run.label));
    }
    if !run.transport_fallback_recovered {
        findings.push(format!("{}:transport_fallback_failed", run.label));
    }
    if !run.rollback_drill_passed {
        findings.push(format!("{}:rollback_drill_failed", run.label));
    }
    findings
}

pub fn evaluate_differential_acceptance(
    input: &DifferentialAcceptanceInput,
) -> DifferentialAcceptanceReceipt {
    let baseline_rate = rate(&input.baseline_without_focusa.cycles, |cycle| {
        cycle.task_success
    });
    let legacy_rate = rate(&input.legacy_focusa.cycles, |cycle| cycle.task_success);
    let adaptive_rate = rate(&input.adaptive_focusa.cycles, |cycle| cycle.task_success);
    let next_rate = rate(&input.adaptive_focusa.cycles, |cycle| {
        cycle.correct_next_action
    });
    let mut findings = Vec::new();
    for run in [
        &input.baseline_without_focusa,
        &input.legacy_focusa,
        &input.adaptive_focusa,
    ] {
        if run.cycles.len() < 3 {
            findings.push(format!("{}:minimum_three_cycles_required", run.label));
        }
        findings.extend(hard_findings(run));
    }
    let floor = baseline_rate.max(legacy_rate) - input.noninferiority_epsilon.abs();
    if adaptive_rate < floor {
        findings.push("adaptive_task_success_inferior".into());
    }
    if next_rate < floor {
        findings.push("adaptive_next_action_inferior".into());
    }
    let before: u64 = input
        .adaptive_focusa
        .cycles
        .iter()
        .map(|cycle| cycle.tokens_before)
        .sum();
    let after: u64 = input
        .adaptive_focusa
        .cycles
        .iter()
        .map(|cycle| cycle.tokens_after)
        .sum();
    let context_release = if before == 0 {
        0.0
    } else {
        (before.saturating_sub(after)) as f64 / before as f64
    };
    let digest = format!(
        "sha256:{}",
        hex::encode(Sha256::digest(
            serde_json::to_vec(input).unwrap_or_default()
        ))
    );
    DifferentialAcceptanceReceipt {
        schema: "focusa.compaction_differential_acceptance_receipt.v1".into(),
        status: if findings.is_empty() {
            "accepted".into()
        } else {
            "blocked".into()
        },
        runtime_segment: input.runtime_segment.clone(),
        provider_strategy: input.provider_strategy.clone(),
        cycles_per_run: input.adaptive_focusa.cycles.len(),
        findings,
        baseline_task_success_rate: baseline_rate,
        legacy_task_success_rate: legacy_rate,
        adaptive_task_success_rate: adaptive_rate,
        adaptive_next_action_rate: next_rate,
        adaptive_context_release_ratio: context_release,
        adaptive_billed_tokens: input
            .adaptive_focusa
            .cycles
            .iter()
            .map(|cycle| cycle.billed_tokens)
            .sum(),
        adaptive_latency_ms: input
            .adaptive_focusa
            .cycles
            .iter()
            .map(|cycle| cycle.latency_ms)
            .sum(),
        evidence_digest: digest,
    }
}
