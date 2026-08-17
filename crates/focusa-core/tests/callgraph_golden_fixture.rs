//! Golden fixture conformance (#294 no-inference gate): the canonical
//! CallGraph definition must validate, digest deterministically, export
//! losslessly, and produce a stable envelope — byte-for-byte across runs.

use focusa_core::callgraph::validate_graph;
use focusa_core::callgraph_export::{
    export_jsonl, export_todo_txt, CallGraphExportProjection,
};
use focusa_core::callgraph_envelope::build_item_envelope;
use focusa_core::callgraph::FocusaCallGraphDefinition;

fn load_golden() -> FocusaCallGraphDefinition {
    let raw = include_str!("fixtures/callgraph-golden.v1.json");
    serde_json::from_str(raw).expect("golden fixture parses")
}

#[test]
fn golden_fixture_validates() {
    let graph = load_golden();
    let report = validate_graph(&graph);
    assert!(report.valid, "golden fixture must validate: {:?}", report.issues);
}

#[test]
fn golden_digest_is_stable() {
    let graph = load_golden();
    let p1 = CallGraphExportProjection::new(graph.clone(), vec![], "jsonl", true, vec![]);
    let p2 = CallGraphExportProjection::new(graph, vec![], "jsonl", true, vec![]);
    assert_eq!(p1.manifest.digest, p2.manifest.digest);
    // Deterministic digest shape: sha256-prefixed, 64 hex chars.
    assert!(p1.manifest.digest.starts_with("sha256:"));
    assert_eq!(p1.manifest.digest.len(), 71); // 7 + 64
    assert_eq!(p1.manifest.digest, p2.manifest.digest);
}

#[test]
fn golden_exports_are_deterministic_and_lossless() {
    let graph = load_golden();
    let p1 = CallGraphExportProjection::new(graph.clone(), vec![], "jsonl", true, vec![]);
    let p2 = CallGraphExportProjection::new(graph, vec![], "jsonl", true, vec![]);
    assert_eq!(export_jsonl(&p1), export_jsonl(&p2));
    assert_eq!(export_jsonl(&p1).lines().count(), 6); // def + 3 frames + 2 edges
    let todo = export_todo_txt(&p1);
    assert!(todo.contains("lossy:true"));
    assert!(todo.contains("dep:review"));
}

#[test]
fn golden_envelope_is_stable() {
    let graph = load_golden();
    let frame = graph.frames.iter().find(|f| f.frame_id == "approve").unwrap();
    let e1 = build_item_envelope(&graph, frame, None);
    let e2 = build_item_envelope(&graph, frame, None);
    assert_eq!(e1.content_digest, e2.content_digest);
    assert_eq!(e1.identity.item_kind, "approval_gate");
    assert!(e1.authority.mutation_confirmation_required);
}
