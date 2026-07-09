//! Append-only Eval Ledger (Spec 114 Phase 1).
//!
//! Endpoints: /v1/evals/runs, /v1/evals/runs/{id}/events, /v1/evals/runs/{id}/complete,
//! /v1/evals/runs/{id}, /v1/evals/runs/compare.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerKind {
    Run,
    Event,
    Complete,
    Read,
    Compare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: String,
    pub kind: LedgerKind,
    pub run_id: String,
    pub arm: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    /// Hash of the previous entry — append-only chain.
    pub prev_hash: String,
    /// Hash of this entry.
    pub entry_hash: String,
    /// Entry payload (JSON-encoded).
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvalLedger {
    /// Append-only chain — keys are monotonic sequence numbers, not UUIDs.
    pub entries: BTreeMap<u64, LedgerEntry>,
    pub runs: BTreeMap<String, EvalRun>,
    pub schema_version: String,
    pub next_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRun {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub arm: String,
    pub model: String,
    pub task_count: u32,
    pub passed: u32,
    pub failed: u32,
    pub notes: Vec<String>,
}

fn kind_str(kind: LedgerKind) -> &'static str {
    match kind {
        LedgerKind::Run => "run",
        LedgerKind::Event => "event",
        LedgerKind::Complete => "complete",
        LedgerKind::Read => "read",
        LedgerKind::Compare => "compare",
    }
}

/// Deterministic FNV-1a 64-bit hash — not StdHash's RandomState.
fn fnv1a_64(s: &str) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

fn compute_entry_hash(prev_hash: &str, kind: LedgerKind, run_id: &str, arm: &str, model: &str, payload: &serde_json::Value) -> String {
    let payload_str = serde_json::to_string(payload).unwrap_or_default();
    let composite = format!("{}|{}|{}|{}|{}|{}",
        prev_hash, kind_str(kind), run_id, arm, model, payload_str);
    let h1 = fnv1a_64(&composite);
    let h2 = fnv1a_64(&format!("{}::{}", composite, h1));
    format!("fnv1a:{:016x}{:016x}", h1, h2)
}

impl EvalLedger {
    pub fn new() -> Self {
        EvalLedger {
            entries: BTreeMap::new(),
            runs: BTreeMap::new(),
            schema_version: "focusa.eval_ledger.v1".to_string(),
            next_seq: 0,
        }
    }

    /// Append a new entry to the ledger. Maintains hash chain.
    pub fn append(&mut self, kind: LedgerKind, run_id: &str, arm: &str, model: &str, payload: serde_json::Value) -> LedgerEntry {
        let prev_hash = self.entries.values().next_back()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(|| "genesis".to_string());
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();
        let entry_hash = compute_entry_hash(&prev_hash, kind, run_id, arm, model, &payload);
        let entry = LedgerEntry {
            id: id.clone(),
            kind,
            run_id: run_id.to_string(),
            arm: arm.to_string(),
            model: model.to_string(),
            created_at: Utc::now(),
            prev_hash,
            entry_hash,
            payload,
        };
        let seq = self.next_seq;
        self.entries.insert(seq, entry.clone());
        self.next_seq += 1;
        entry
    }

    /// Compare two runs by id and return deltas.
    pub fn compare(&self, run_a: &str, run_b: &str) -> serde_json::Value {
        let a = self.runs.get(run_a);
        let b = self.runs.get(run_b);
        match (a, b) {
            (Some(a), Some(b)) => serde_json::json!({
                "run_a": a,
                "run_b": b,
                "delta": {
                    "pass_rate_a": if a.task_count > 0 { a.passed as f64 / a.task_count as f64 } else { 0.0 },
                    "pass_rate_b": if b.task_count > 0 { b.passed as f64 / b.task_count as f64 } else { 0.0 },
                },
            }),
            _ => serde_json::json!({"error": "run_not_found", "run_a": run_a, "run_b": run_b}),
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn run_count(&self) -> usize {
        self.runs.len()
    }
}
