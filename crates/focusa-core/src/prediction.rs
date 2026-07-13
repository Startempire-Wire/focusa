//! Typed prediction state values stored inside scoped CRDT records.

use crate::types::TrajectoryLadderContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PredictionOntologyContext {
    pub object_refs: Vec<String>,
    pub action_refs: Vec<String>,
    pub tool_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub relation_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionOutcomeCapture {
    pub mode: String,
    pub matched_by: String,
    #[serde(default)]
    pub context_refs: Vec<String>,
    #[serde(default)]
    pub ontology_context: PredictionOntologyContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionValue {
    pub prediction_type: String,
    #[serde(default)]
    pub context_refs: Vec<String>,
    #[serde(default)]
    pub ontology_context: PredictionOntologyContext,
    pub predicted_outcome: String,
    pub confidence: f64,
    pub recommended_action: String,
    pub why: String,
    #[serde(default)]
    pub trajectory: Option<TrajectoryLadderContext>,
    #[serde(default)]
    pub actual_outcome: Option<String>,
    #[serde(default)]
    pub evaluated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub learning_signal_ref: Option<String>,
    #[serde(default)]
    pub outcome_capture: Option<PredictionOutcomeCapture>,
}
