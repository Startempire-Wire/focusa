//! Spec144 truthful Semantic Pair projection for terminal surfaces.
use serde::{Deserialize, Serialize};

pub const TRUTH_STATES: &[&str] = &[
    "schema_only", "pack_missing", "migration_required", "verification_required",
    "verification_blocked", "operator_required", "unsupported_future_definition",
    "writer_blocked", "degraded", "stale", "conflicted", "quarantined",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticOperation {
    pub operation_id: String,
    pub kind: String,
    pub availability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticPairProjection {
    pub state: String,
    #[serde(default)] pub operations: Vec<SemanticOperation>,
    #[serde(default)] pub obligations: usize,
    #[serde(default)] pub findings: usize,
    #[serde(default)] pub settlement: Option<String>,
    #[serde(default)] pub replay: Option<String>,
    #[serde(default)] pub recovery: Option<String>,
}

/// TUI is observational: mutations stay visible with explicit unsupported truth.
pub fn lines(model: &SemanticPairProjection, width: usize) -> Vec<String> {
    let state = if TRUTH_STATES.contains(&model.state.as_str()) { &model.state } else { "schema_only" };
    let mut out = vec![
        fit(format!("SEMANTIC PAIR · {state}"), width),
        fit(format!("obligations {} · findings {} · settlement {} · replay {}",
            model.obligations, model.findings,
            model.settlement.as_deref().unwrap_or("unsettled"),
            model.replay.as_deref().unwrap_or("not_requested")), width),
    ];
    if let Some(recovery) = &model.recovery {
        out.push(fit(format!("recovery · {recovery}"), width));
    }
    out.extend(model.operations.iter().map(|op| {
        let support = if op.kind == "mutation" { "unsupported on TUI (read-only)" } else { op.availability.as_str() };
        fit(format!("{} · {} · {}", op.operation_id, op.kind, support), width)
    }));
    out
}

fn fit(mut value: String, width: usize) -> String {
    if width == 0 { return String::new(); }
    if value.chars().count() <= width { return value; }
    value = value.chars().take(width.saturating_sub(1)).collect();
    value.push('…');
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutation_is_visible_but_never_claimed_supported() {
        let model = SemanticPairProjection { state: "quarantined".into(), operations: vec![SemanticOperation {
            operation_id: "semantic_pair.settlement.commit".into(), kind: "mutation".into(), availability: "writer_blocked".into(),
        }], obligations: 2, findings: 1, settlement: None, replay: None, recovery: Some("operator_required".into()) };
        let rendered = lines(&model, 120).join("\n");
        assert!(rendered.contains("quarantined"));
        assert!(rendered.contains("unsupported on TUI"));
    }
}
