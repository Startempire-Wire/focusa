//! Benchmark metrics (Spec 113).
//!
//! Agent Power Index, Focusa Uplift Score, Operator Burden Reduction,
//! Groundedness, Hallucination Rate, Tool-Call Accuracy, Pass^N, Time-Horizon.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPowerIndex {
    /// Raw success rate across the task pool.
    pub success_rate: f64,
    /// Weighted score by difficulty.
    pub weighted_score: f64,
    /// Number of tasks attempted.
    pub tasks_attempted: u32,
    /// Number of tasks completed successfully.
    pub tasks_completed: u32,
    /// Computed agent power index (0..=1).
    pub index: f64,
}

impl AgentPowerIndex {
    pub fn from_outcomes(outcomes: &[(bool, u8)]) -> Self {
        let attempted = outcomes.len() as u32;
        let completed = outcomes.iter().filter(|(s, _)| *s).count() as u32;
        let success_rate = if attempted == 0 { 0.0 } else { completed as f64 / attempted as f64 };
        let weight: f64 = outcomes.iter().map(|(s, d)| if *s { *d as f64 } else { 0.0 }).sum();
        let weight_max: f64 = outcomes.iter().map(|(_, d)| *d as f64).sum();
        let weighted_score = if weight_max == 0.0 { 0.0 } else { weight / weight_max };
        let index = success_rate * 0.6 + weighted_score * 0.4;
        AgentPowerIndex { success_rate, weighted_score, tasks_attempted: attempted, tasks_completed: completed, index }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusaUpliftScore {
    /// (FullFocusa score) - (NoFocusa score), normalized to 0..=1.
    pub uplift: f64,
    /// Absolute improvement in pass rate.
    pub pass_rate_delta: f64,
    /// Tasks where Focusa helps (FullFocusa pass, NoFocusa fail).
    pub tasks_helped: u32,
    /// Tasks where Focusa hurts (FullFocusa fail, NoFocusa pass).
    pub tasks_hurt: u32,
    /// Tasks where both pass (neutral).
    pub tasks_neutral: u32,
}

impl FocusaUpliftScore {
    /// Compute uplift by comparing FullFocusa vs NoFocusa outcomes.
    pub fn from_comparison(full: &[bool], baseline: &[bool]) -> Self {
        let mut helped = 0;
        let mut hurt = 0;
        let mut neutral = 0;
        let mut full_pass = 0;
        let mut base_pass = 0;
        let n = full.len().min(baseline.len());
        for i in 0..n {
            if full[i] { full_pass += 1; }
            if baseline[i] { base_pass += 1; }
            match (full[i], baseline[i]) {
                (true, false) => helped += 1,
                (false, true) => hurt += 1,
                (true, true) | (false, false) => neutral += 1,
            }
        }
        let pass_rate_delta = if n == 0 { 0.0 } else {
            (full_pass as f64 - base_pass as f64) / n as f64
        };
        // Normalize to 0..=1
        let uplift = (pass_rate_delta + 1.0) / 2.0;
        FocusaUpliftScore { uplift, pass_rate_delta, tasks_helped: helped, tasks_hurt: hurt, tasks_neutral: neutral }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorBurdenReduction {
    /// Wall-clock minutes saved per task.
    pub minutes_saved_per_task: f64,
    /// Median recovery time without Focusa (minutes).
    pub baseline_recovery_minutes: f64,
    /// Median recovery time with Focusa (minutes).
    pub focusa_recovery_minutes: f64,
    /// Aggregate reduction (0..=1).
    pub reduction: f64,
}

impl OperatorBurdenReduction {
    pub fn from_recovery(baseline: f64, focusa: f64, _tasks: u32) -> Self {
        let reduction = if baseline == 0.0 { 0.0 } else { 1.0 - focusa / baseline };
        OperatorBurdenReduction {
            minutes_saved_per_task: baseline - focusa,
            baseline_recovery_minutes: baseline,
            focusa_recovery_minutes: focusa,
            reduction: reduction.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundednessScore {
    /// 0..=1 — proportion of claims backed by evidence.
    pub score: f64,
    /// Total claims observed.
    pub claims_total: u32,
    /// Claims backed by evidence.
    pub claims_grounded: u32,
}

impl GroundednessScore {
    pub fn from_observations(claims_total: u32, claims_grounded: u32) -> Self {
        let score = if claims_total == 0 { 0.0 } else {
            claims_grounded as f64 / claims_total as f64
        };
        GroundednessScore { score, claims_total, claims_grounded }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallucinationRate {
    /// 0..=1 — proportion of responses containing hallucinations.
    pub rate: f64,
    /// Total responses observed.
    pub responses_total: u32,
    /// Responses containing hallucinations.
    pub responses_with_hallucination: u32,
}

impl HallucinationRate {
    pub fn from_observations(responses_total: u32, hallucinated: u32) -> Self {
        let rate = if responses_total == 0 { 0.0 } else {
            hallucinated as f64 / responses_total as f64
        };
        HallucinationRate { rate, responses_total, responses_with_hallucination: hallucinated }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallAccuracy {
    /// 0..=1 — proportion of tool calls that match the expected contract.
    pub accuracy: f64,
    /// Total tool calls observed.
    pub calls_total: u32,
    /// Tool calls that matched the contract.
    pub calls_correct: u32,
}

impl ToolCallAccuracy {
    pub fn from_observations(calls_total: u32, calls_correct: u32) -> Self {
        let accuracy = if calls_total == 0 { 0.0 } else {
            calls_correct as f64 / calls_total as f64
        };
        ToolCallAccuracy { accuracy, calls_total, calls_correct }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassAtN {
    /// Probability that at least one of n attempts succeeded.
    pub pass_at_n: f64,
    /// Number of attempts (n).
    pub n: u32,
    /// Number of unique successful attempts.
    pub k_success: u32,
    /// Number of unique total attempts.
    pub k_total: u32,
}

impl PassAtN {
    /// Standard estimator: 1 - C(k_total - k_success, n) / C(k_total, n).
    pub fn from_observations(n: u32, k_success: u32, k_total: u32) -> Self {
        let pass_at_n = if k_total == 0 || n == 0 || k_total < n {
            0.0
        } else if k_total - k_success >= n {
            // Safe combination
            let total = comb(k_total, n);
            let fail = comb(k_total - k_success, n);
            1.0 - (fail / total)
        } else {
            1.0
        };
        PassAtN { pass_at_n, n, k_success, k_total }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeHorizon {
    /// Median time to complete a task (seconds).
    pub median_seconds: f64,
    /// 95th percentile time (seconds).
    pub p95_seconds: f64,
    /// Tasks measured.
    pub tasks_measured: u32,
}

impl TimeHorizon {
    pub fn from_observations(times_seconds: &[f64]) -> Self {
        if times_seconds.is_empty() {
            return TimeHorizon { median_seconds: 0.0, p95_seconds: 0.0, tasks_measured: 0 };
        }
        let mut sorted: Vec<f64> = times_seconds.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_idx = sorted.len() / 2;
        let p95_idx = (sorted.len() as f64 * 0.95).floor() as usize;
        TimeHorizon {
            median_seconds: sorted[median_idx.min(sorted.len() - 1)],
            p95_seconds: sorted[p95_idx.min(sorted.len() - 1)],
            tasks_measured: sorted.len() as u32,
        }
    }
}

/// Compute n choose k using u128 to avoid overflow for our small n.
fn comb(n: u32, k: u32) -> f64 {
    if k > n { return 0.0; }
    let k = k.min(n - k);
    let mut num: u128 = 1;
    let mut den: u128 = 1;
    for i in 1..=k {
        num = num.saturating_mul((n - k + i) as u128);
        den = den.saturating_mul(i as u128);
    }
    if den == 0 { 1.0 } else { num as f64 / den as f64 }
}
