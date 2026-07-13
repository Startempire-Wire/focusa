//! Spec 130 bounded subagent-result intake.

use axum::{Json, Router, http::StatusCode, routing::post};
use focusa_core::scope_safety::classify_project_root;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

const MAX_SUMMARY_CHARS: usize = 2000;
const MAX_ITEMS: usize = 32;
const MAX_ITEM_CHARS: usize = 512;

#[derive(Debug, Clone, Deserialize)]
pub struct SubagentScope {
    pub project_root: String,
    pub continuity_id: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubagentResultRequest {
    pub task: String,
    pub scope: SubagentScope,
    pub summary: String,
    #[serde(default)]
    pub inspected_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub changed_files: Vec<String>,
    #[serde(default)]
    pub active_blockers: Vec<String>,
    pub confidence: String,
    #[serde(default)]
    pub omitted_raw_refs: Vec<String>,
    pub recommended_next: Option<String>,
    #[serde(default)]
    pub must_not_infer: Vec<String>,
    #[serde(default)]
    pub rehydrate_refs: Vec<String>,
}

fn bounded(value: &str, max: usize) -> String {
    value.trim().chars().take(max).collect()
}

fn bounded_items(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| bounded(value, MAX_ITEM_CHARS))
        .filter(|value| !value.is_empty())
        .take(MAX_ITEMS)
        .collect()
}

fn validate_and_render(body: &SubagentResultRequest) -> Result<Value, Value> {
    if !classify_project_root(&body.scope.project_root).is_safe() {
        return Err(json!({
            "schema": "focusa.subagent_result_error.v1",
            "status": "blocked",
            "failure_class": "unsafe_project_scope"
        }));
    }
    if body.scope.continuity_id.trim().is_empty() {
        return Err(json!({
            "schema": "focusa.subagent_result_error.v1",
            "status": "blocked",
            "failure_class": "continuity_id_missing"
        }));
    }
    if !matches!(body.confidence.as_str(), "low" | "medium" | "high") {
        return Err(json!({
            "schema": "focusa.subagent_result_error.v1",
            "status": "blocked",
            "failure_class": "invalid_confidence"
        }));
    }
    if body.summary.chars().count() > MAX_SUMMARY_CHARS {
        return Err(json!({
            "schema": "focusa.subagent_result_error.v1",
            "status": "blocked",
            "failure_class": "unbounded_summary",
            "max_chars": MAX_SUMMARY_CHARS
        }));
    }
    let omitted_raw_refs = bounded_items(&body.omitted_raw_refs);
    let rehydrate_refs = bounded_items(&body.rehydrate_refs);
    if !omitted_raw_refs.is_empty() && rehydrate_refs.is_empty() {
        return Err(json!({
            "schema": "focusa.subagent_result_error.v1",
            "status": "blocked",
            "failure_class": "omitted_raw_without_rehydrate_ref"
        }));
    }
    Ok(json!({
        "schema": "focusa.subagent_result.v1",
        "canonical": false,
        "advisory": true,
        "task": bounded(&body.task, 500),
        "scope": {
            "project_root": body.scope.project_root,
            "continuity_id": body.scope.continuity_id,
            "session_id": body.scope.session_id
        },
        "summary": bounded(&body.summary, MAX_SUMMARY_CHARS),
        "inspected_refs": bounded_items(&body.inspected_refs),
        "evidence_refs": bounded_items(&body.evidence_refs),
        "changed_files": bounded_items(&body.changed_files),
        "active_blockers": bounded_items(&body.active_blockers),
        "confidence": body.confidence,
        "omitted_raw_refs": omitted_raw_refs,
        "recommended_next": body.recommended_next.as_deref().map(|value| bounded(value, 500)),
        "must_not_infer": bounded_items(&body.must_not_infer),
        "rehydrate_refs": rehydrate_refs,
        "authority": "advisory subagent result; canonical project, Trajectory, Workpoint, and evidence remain authoritative"
    }))
}

async fn intake(
    Json(body): Json<SubagentResultRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    validate_and_render(&body)
        .map(Json)
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, Json(error)))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/subagent/result", post(intake))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> SubagentResultRequest {
        SubagentResultRequest {
            task: "inspect bounded surface".into(),
            scope: SubagentScope {
                project_root: "/tmp/safe-project".into(),
                continuity_id: "focusa-cont-test".into(),
                session_id: Some("pi-test".into()),
            },
            summary: "bounded result".into(),
            inspected_refs: vec!["src/lib.rs".into()],
            evidence_refs: vec!["test:pass".into()],
            changed_files: vec![],
            active_blockers: vec![],
            confidence: "high".into(),
            omitted_raw_refs: vec![],
            recommended_next: Some("continue".into()),
            must_not_infer: vec!["release authority".into()],
            rehydrate_refs: vec!["focusa_traverse".into()],
        }
    }

    #[test]
    fn valid_result_is_bounded_and_advisory() {
        let value = validate_and_render(&request()).expect("valid result");
        assert_eq!(value["schema"], "focusa.subagent_result.v1");
        assert_eq!(value["canonical"], false);
        assert_eq!(value["advisory"], true);
    }

    #[test]
    fn raw_omission_requires_rehydrate_reference() {
        let mut request = request();
        request.omitted_raw_refs = vec!["handle:raw-log".into()];
        request.rehydrate_refs.clear();
        let error = validate_and_render(&request).expect_err("must reject");
        assert_eq!(error["failure_class"], "omitted_raw_without_rehydrate_ref");
    }

    #[test]
    fn unbounded_summary_is_rejected() {
        let mut request = request();
        request.summary = "x".repeat(MAX_SUMMARY_CHARS + 1);
        let error = validate_and_render(&request).expect_err("must reject");
        assert_eq!(error["failure_class"], "unbounded_summary");
    }
}
