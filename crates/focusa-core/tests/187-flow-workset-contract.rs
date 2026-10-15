//! Synthetic source-authoring proof only, not admission or installed flow.
use focusa_core::callgraph::{FocusaCallGraphDefinition, validate_graph};
use focusa_core::workset_ledger::CompletionContract;

#[test]
fn spec187_projection_uses_the_actual_callgraph_and_workset_contracts() {
    let graph: FocusaCallGraphDefinition = serde_json::from_str(include_str!(
        "../../../docs/contracts/spec187/callgraph.structural-fixture.v1.json"
    ))
    .expect("native Spec155 definition must deserialize");
    let report = validate_graph(&graph);
    assert!(report.valid, "{:#?}", report.issues);
    assert_eq!(graph.frames.len(), 19);
    assert_eq!(graph.entry_frame_ids, ["FLOW-00"]);
    assert_eq!(graph.created_by.authority_kind, "synthetic_fixture");
    assert!(graph.frames.iter().all(|f| f.execution_binding.is_none()));
    let profile: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/contracts/spec187/release-workset.v1.json"
    ))
    .unwrap();
    let contract: CompletionContract =
        serde_json::from_value(profile["completion_contract"].clone()).unwrap();
    assert_eq!(contract.required_requirement_ids.len(), 19);
    for requirement in &contract.required_requirement_ids {
        assert!(
            graph
                .frames
                .iter()
                .any(|f| f.acceptance.acceptance_atoms.contains(requirement))
        );
    }
}
