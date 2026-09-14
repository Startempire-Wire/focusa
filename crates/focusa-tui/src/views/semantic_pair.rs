//! Spec144 truthful Semantic Pair projection for terminal surfaces.
use serde::{Deserialize, Serialize};

pub const TRUTH_STATES: &[&str] = &[
    "supported",
    "schema_only",
    "pack_missing",
    "migration_required",
    "verification_required",
    "verification_blocked",
    "operator_required",
    "unsupported_future_definition",
    "writer_blocked",
    "degraded",
    "stale",
    "conflicted",
    "quarantined",
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
    #[serde(default)]
    pub operations: Vec<SemanticOperation>,
    #[serde(default)]
    pub obligations: usize,
    #[serde(default)]
    pub findings: usize,
    #[serde(default)]
    pub settlement: Option<String>,
    #[serde(default)]
    pub replay: Option<String>,
    #[serde(default)]
    pub recovery: Option<String>,
}

/// TUI is observational: every operation stays visible with daemon-reported truth.
pub fn lines(model: &SemanticPairProjection, width: usize) -> Vec<String> {
    let state = if TRUTH_STATES.contains(&model.state.as_str()) {
        &model.state
    } else {
        "schema_only"
    };
    let mut out = vec![
        fit(format!("SEMANTIC PAIR · {state}"), width),
        fit(
            format!(
                "obligations {} · findings {} · settlement {} · replay {}",
                model.obligations,
                model.findings,
                model.settlement.as_deref().unwrap_or("unsettled"),
                model.replay.as_deref().unwrap_or("not_requested")
            ),
            width,
        ),
    ];
    if let Some(recovery) = &model.recovery {
        out.push(fit(format!("recovery · {recovery}"), width));
    }
    out.extend(model.operations.iter().map(|op| {
        fit(
            format!("{} · {} · {}", op.operation_id, op.kind, op.availability),
            width,
        )
    }));
    out
}

fn fit(mut value: String, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if value.chars().count() <= width {
        return value;
    }
    value = value.chars().take(width.saturating_sub(1)).collect();
    value.push('…');
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutation_is_visible_with_daemon_reported_availability() {
        let model = SemanticPairProjection {
            state: "supported".into(),
            operations: vec![SemanticOperation {
                operation_id: "semantic_pair.settlement.commit".into(),
                kind: "mutation".into(),
                availability: "supported".into(),
            }],
            obligations: 2,
            findings: 1,
            settlement: None,
            replay: None,
            recovery: Some("operator_required".into()),
        };
        let rendered = lines(&model, 120).join("\n");
        assert!(rendered.contains("supported"));
        assert!(rendered.contains("semantic_pair.settlement.commit · mutation · supported"));
    }
}
