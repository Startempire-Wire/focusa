//! Benchmark reports + measured-claim templates (Spec 113).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredClaim {
    /// What the claim is about (e.g., "agent_power_index", "uplift_score").
    pub metric: String,
    /// The claimed value.
    pub value: f64,
    /// Confidence interval [low, high].
    pub confidence_interval: [f64; 2],
    /// Statistical confidence level (e.g., 0.95).
    pub confidence_level: f64,
    /// Number of measurements supporting the claim.
    pub n: u32,
    /// Sample standard deviation.
    pub stddev: f64,
    /// Source: which benchmark run(s) this claim came from.
    pub source_runs: Vec<String>,
    /// Verification method (e.g., "bootstrap", "wilson", "t_test").
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportArtifact {
    pub kind: String,
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub runs: Vec<String>,
    pub claims: BTreeMap<String, MeasuredClaim>,
    pub artifacts: Vec<ReportArtifact>,
    pub schema_version: String,
    pub notes: Vec<String>,
}

impl BenchmarkReport {
    pub fn new(title: impl Into<String>) -> Self {
        BenchmarkReport {
            id: uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            title: title.into(),
            created_at: Utc::now(),
            runs: Vec::new(),
            claims: BTreeMap::new(),
            artifacts: Vec::new(),
            schema_version: "focusa.benchmark_report.v1".to_string(),
            notes: Vec::new(),
        }
    }

    pub fn add_claim(&mut self, metric: impl Into<String>, value: f64, ci_low: f64, ci_high: f64, n: u32) {
        let claim = MeasuredClaim {
            metric: metric.into(),
            value,
            confidence_interval: [ci_low, ci_high],
            confidence_level: 0.95,
            n,
            stddev: 0.0,
            source_runs: Vec::new(),
            method: "wilson".to_string(),
        };
        self.claims.insert(claim.metric.clone(), claim);
    }
}