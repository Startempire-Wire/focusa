use focusa_core::callgraph::FocusaCallGraphDefinition;

use super::callgraph::validation_response;

fn golden_graph() -> FocusaCallGraphDefinition {
    serde_json::from_str(include_str!(
        "../../../focusa-core/tests/fixtures/callgraph-golden.v1.json"
    ))
    .expect("golden CallGraph fixture must deserialize")
}

#[test]
fn golden_graph_returns_typed_valid_envelope() {
    let graph = golden_graph();
    let response = validation_response(&graph);

    assert_eq!(response["status"], "valid");
    assert_eq!(response["valid"], true);
    assert_eq!(response["canonical"], true);
    assert_eq!(response["graph_id"], graph.graph_id);
    assert_eq!(response["revision"], graph.revision);
    assert_eq!(response["issues"].as_array().map(Vec::len), Some(0));
}

#[test]
fn invalid_graph_preserves_deterministic_validation_issues() {
    let mut graph = golden_graph();
    graph.entry_frame_ids = vec!["missing-entry".to_string()];
    let response = validation_response(&graph);

    assert_eq!(response["status"], "invalid");
    assert_eq!(response["valid"], false);
    let issues = response["issues"].as_array().expect("typed issues");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["path"], "entry_frame_ids");
    assert!(
        issues[0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("missing-entry")
    );
}
