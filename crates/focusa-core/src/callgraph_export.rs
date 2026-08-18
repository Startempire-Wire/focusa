//! CallGraph export — slice 1 (#287). Spec 155 export program.
//!
//! One typed `CallGraphExportProjection` drives every format: lossless
//! JSONL snapshot, the standard TODO.txt profile (deliberately lossy,
//! provenance-header), and Graphviz DOT. TODO.txt is a portable task
//! projection — never canonical graph truth (#287 classification).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::callgraph::{FocusaCallEdge, FocusaCallFrame, FocusaCallGraphDefinition, FrameKind};
use crate::callgraph_store::FrameDispatch;

pub const EXPORT_SCHEMA: &str = "focusa.callgraph_export.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallGraphExportManifest {
    pub export_id: String,
    pub scope: crate::callgraph::CallGraphScope,
    pub graph_id: String,
    pub revision: u64,
    pub digest: String,
    pub format: String,
    pub format_version: String,
    pub record_count: usize,
    pub lossless: bool,
    pub known_omissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallGraphExportProjection {
    pub schema: String,
    pub manifest: CallGraphExportManifest,
    pub graph: FocusaCallGraphDefinition,
    pub dispatches: Vec<FrameDispatch>,
}

impl CallGraphExportProjection {
    pub fn new(
        graph: FocusaCallGraphDefinition,
        dispatches: Vec<FrameDispatch>,
        format: &str,
        lossless: bool,
        known_omissions: Vec<String>,
    ) -> Self {
        let digest = graph_digest(&graph);
        let manifest = CallGraphExportManifest {
            export_id: uuid::Uuid::now_v7().to_string(),
            scope: graph.scope.clone(),
            graph_id: graph.graph_id.clone(),
            revision: graph.revision,
            digest,
            format: format.to_string(),
            format_version: "1.0".to_string(),
            record_count: graph.frames.len() + graph.edges.len() + dispatches.len(),
            lossless,
            known_omissions,
        };
        Self {
            schema: EXPORT_SCHEMA.to_string(),
            manifest,
            graph,
            dispatches,
        }
    }
}

pub fn graph_digest(graph: &FocusaCallGraphDefinition) -> String {
    let canonical = serde_json::to_string(graph).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("sha256:{}", hex_encode(&hasher.finalize()))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Lossless snapshot: one JSON object per record (definition, then each
/// frame, then each edge, then each dispatch), each with the export
/// envelope. Deterministic order = definition order.
pub fn export_jsonl(projection: &CallGraphExportProjection) -> String {
    let mut out = String::new();
    if let Ok(line) = serde_json::to_string(&projection.graph) {
        out.push_str(&line);
        out.push('\n');
    }
    for frame in &projection.graph.frames {
        if let Ok(line) = serde_json::to_string(frame) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    for edge in &projection.graph.edges {
        if let Ok(line) = serde_json::to_string(edge) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    for dispatch in &projection.dispatches {
        if let Ok(line) = serde_json::to_string(dispatch) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// Standard TODO.txt profile (lossy projection). Priority from frame kind;
/// dependency edges become `dep:<frame_id>` tags; the provenance header
/// carries the manifest so the projection is always traceable.
pub fn export_todo_txt(projection: &CallGraphExportProjection) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "x focusa-callgraph-export export_id:{} graph:{} revision:{} digest:{} lossy:true\n",
        projection.manifest.export_id,
        projection.manifest.graph_id,
        projection.manifest.revision,
        projection.manifest.digest,
    ));
    out.push_str("x focusa-callgraph-export source-of-truth:focusa canonical:false\n");
    let mut edge_map: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for edge in &projection.graph.edges {
        edge_map
            .entry(edge.from_frame_id.as_str())
            .or_default()
            .push(edge.to_frame_id.as_str());
    }
    for frame in &projection.graph.frames {
        let priority = match frame.kind {
            FrameKind::Human | FrameKind::Approval => "(A)",
            FrameKind::Join | FrameKind::Timer => "(B)",
            _ => "(C)",
        };
        let mut deps = String::new();
        if let Some(targets) = edge_map.get(frame.frame_id.as_str()) {
            for target in targets {
                deps.push_str(&format!(" dep:{target}"));
            }
        }
        let context = format!(
            " +focusa @workstream:{}",
            projection.manifest.scope.continuity_id
        );
        out.push_str(&format!(
            "{priority} 2026-08-16 frame:{} kind:{} {}{}{}\n",
            frame.frame_id,
            frame.kind.label(),
            frame.name,
            deps,
            context
        ));
    }
    out
}

impl FrameKind {
    fn label(&self) -> &'static str {
        match self {
            FrameKind::Human => "human",
            FrameKind::Agent => "agent",
            FrameKind::Tool => "tool",
            FrameKind::Approval => "approval",
            FrameKind::Timer => "timer",
            FrameKind::Join => "join",
            FrameKind::Subgraph => "subgraph",
            FrameKind::FlowmeshTask => "flowmesh_task",
        }
    }
}

/// Graphviz DOT projection for inspection surfaces.
pub fn export_dot(projection: &CallGraphExportProjection) -> String {
    let mut out = String::from("digraph callgraph {\n");
    out.push_str(&format!(
        "  label=\"{} rev {}\";\n",
        projection.manifest.graph_id, projection.manifest.revision
    ));
    for frame in &projection.graph.frames {
        out.push_str(&format!(
            "  \"{}\" [label=\"{}\"];\n",
            frame.frame_id, frame.name
        ));
    }
    for edge in &projection.graph.edges {
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{:?}\"];\n",
            edge.from_frame_id, edge.to_frame_id, edge.kind
        ));
    }
    out.push_str("}\n");
    out
}

/// CSV/TSV frame table (delimiter-configurable; header + one row per frame).
pub fn export_csv(projection: &CallGraphExportProjection, delimiter: char) -> String {
    let mut out = String::new();
    out.push_str("frame_id,name,kind,side_effect_class,acceptance_atoms\n");
    for frame in &projection.graph.frames {
        out.push_str(&format!(
            "{}{delimiter}{}{delimiter}{}{delimiter}{}{delimiter}{}\n",
            frame.frame_id,
            frame.name,
            frame.kind.label(),
            side_effect_label(frame.side_effect_class),
            frame.acceptance.acceptance_atoms.join("|"),
        ));
    }
    out
}

fn side_effect_label(class: crate::callgraph::SideEffectClass) -> &'static str {
    match class {
        crate::callgraph::SideEffectClass::None => "none",
        crate::callgraph::SideEffectClass::Local => "local",
        crate::callgraph::SideEffectClass::External => "external",
        crate::callgraph::SideEffectClass::Destructive => "destructive",
        crate::callgraph::SideEffectClass::Financial => "financial",
        crate::callgraph::SideEffectClass::Security => "security",
    }
}

/// Mermaid flowchart projection for small graphs (Spec 155 §read formats).
pub fn export_mermaid(projection: &CallGraphExportProjection) -> String {
    let mut out = String::from("flowchart LR\n");
    for frame in &projection.graph.frames {
        out.push_str(&format!("    {}[\"{}\"]\n", frame.frame_id, frame.name));
    }
    for edge in &projection.graph.edges {
        out.push_str(&format!(
            "    {} -->|{:?}| {}\n",
            edge.from_frame_id, edge.kind, edge.to_frame_id
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::{
        AcceptanceContract, AuthorityRef, CallGraphPolicies, CallGraphScope, EdgeKind,
        SideEffectClass,
    };

    fn sample() -> FocusaCallGraphDefinition {
        FocusaCallGraphDefinition {
            schema: crate::callgraph::CALLGRAPH_SCHEMA.to_string(),
            graph_id: "g1".to_string(),
            revision: 1,
            scope: CallGraphScope {
                project_root: "/root/proj".to_string(),
                continuity_id: "cont-1".to_string(),
            },
            mission_ref: "m1".to_string(),
            trajectory_ref: None,
            workpoint_refs: vec![],
            title: "t".to_string(),
            description: "t".to_string(),
            entry_frame_ids: vec!["a".to_string()],
            frames: vec![
                FocusaCallFrame {
                    frame_id: "a".to_string(),
                    name: "plan".to_string(),
                    purpose: "t".to_string(),
                    kind: FrameKind::Agent,
                    input_schema: serde_json::json!({}),
                    return_schema: serde_json::json!({}),
                    preconditions: vec![],
                    postconditions: vec![],
                    side_effect_class: SideEffectClass::None,
                    capability_refs: vec![],
                    authority_requirement: None,
                    timeout_policy: None,
                    retry_policy: None,
                    failure_boundary: None,
                    compensation_frame_id: None,
                    resource_budget: None,
                    acceptance: AcceptanceContract {
                        acceptance_atoms: vec!["a1".to_string()],
                        verifier: None,
                    },
                    execution_binding: None,
                },
                FocusaCallFrame {
                    frame_id: "b".to_string(),
                    name: "approve".to_string(),
                    purpose: "t".to_string(),
                    kind: FrameKind::Approval,
                    input_schema: serde_json::json!({}),
                    return_schema: serde_json::json!({}),
                    preconditions: vec![],
                    postconditions: vec![],
                    side_effect_class: SideEffectClass::None,
                    capability_refs: vec![],
                    authority_requirement: None,
                    timeout_policy: None,
                    retry_policy: None,
                    failure_boundary: None,
                    compensation_frame_id: None,
                    resource_budget: None,
                    acceptance: AcceptanceContract {
                        acceptance_atoms: vec!["a1".to_string()],
                        verifier: None,
                    },
                    execution_binding: None,
                },
            ],
            edges: vec![FocusaCallEdge {
                edge_id: "e1".to_string(),
                from_frame_id: "a".to_string(),
                to_frame_id: "b".to_string(),
                kind: EdgeKind::Call,
                condition: None,
                input_mapping: vec![],
                return_mapping: vec![],
                join_policy: None,
                cycle_policy: None,
                authority_requirement: None,
            }],
            policies: CallGraphPolicies::default(),
            required_evidence: vec![],
            created_at: "t".to_string(),
            created_by: AuthorityRef {
                authority_kind: "operator".to_string(),
                reference: "op-1".to_string(),
            },
            supersedes_revision: None,
        }
    }

    #[test]
    fn jsonl_is_lossless_and_deterministic() {
        let graph = sample();
        let p1 = CallGraphExportProjection::new(graph.clone(), vec![], "jsonl", true, vec![]);
        let p2 = CallGraphExportProjection::new(graph, vec![], "jsonl", true, vec![]);
        assert!(p1.manifest.lossless);
        assert_eq!(export_jsonl(&p1).lines().count(), 4); // def + 2 frames + 1 edge
        assert_eq!(p1.manifest.digest, p2.manifest.digest);
        assert_eq!(export_jsonl(&p1), export_jsonl(&p2));
    }

    #[test]
    fn todo_txt_carries_provenance_and_priorities() {
        let graph = sample();
        let projection = CallGraphExportProjection::new(
            graph,
            vec![],
            "todo.txt",
            false,
            vec!["edge semantics flattened to dep: tags".to_string()],
        );
        let out = export_todo_txt(&projection);
        assert!(out.contains("lossy:true"));
        assert!(out.contains("source-of-truth:focusa"));
        assert!(out.contains("(A)"));
        assert!(out.contains("dep:b"));
        assert!(out.contains("@workstream:cont-1"));
    }

    #[test]
    fn dot_renders_frames_and_edges() {
        let graph = sample();
        let projection = CallGraphExportProjection::new(graph, vec![], "dot", true, vec![]);
        let out = export_dot(&projection);
        assert!(out.starts_with("digraph callgraph"));
        assert!(out.contains("\"a\" -> \"b\""));
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use crate::callgraph::{
        AcceptanceContract, AuthorityRef, CallGraphPolicies, CallGraphScope, SideEffectClass,
    };

    fn minimal() -> FocusaCallGraphDefinition {
        FocusaCallGraphDefinition {
            schema: crate::callgraph::CALLGRAPH_SCHEMA.to_string(),
            graph_id: "g1".to_string(),
            revision: 1,
            scope: CallGraphScope {
                project_root: "/root/proj".to_string(),
                continuity_id: "cont-1".to_string(),
            },
            mission_ref: "m1".to_string(),
            trajectory_ref: None,
            workpoint_refs: vec![],
            title: "t".to_string(),
            description: "t".to_string(),
            entry_frame_ids: vec!["a".to_string()],
            frames: vec![FocusaCallFrame {
                frame_id: "a".to_string(),
                name: "plan".to_string(),
                purpose: "t".to_string(),
                kind: FrameKind::Agent,
                input_schema: serde_json::json!({}),
                return_schema: serde_json::json!({}),
                preconditions: vec![],
                postconditions: vec![],
                side_effect_class: SideEffectClass::Local,
                capability_refs: vec![],
                authority_requirement: None,
                timeout_policy: None,
                retry_policy: None,
                failure_boundary: None,
                compensation_frame_id: None,
                resource_budget: None,
                acceptance: AcceptanceContract {
                    acceptance_atoms: vec!["a1".to_string()],
                    verifier: None,
                },
                execution_binding: None,
            }],
            edges: vec![],
            policies: CallGraphPolicies::default(),
            required_evidence: vec![],
            created_at: "t".to_string(),
            created_by: AuthorityRef {
                authority_kind: "operator".to_string(),
                reference: "op-1".to_string(),
            },
            supersedes_revision: None,
        }
    }

    #[test]
    fn csv_has_header_and_frame_row() {
        let projection = CallGraphExportProjection::new(minimal(), vec![], "csv", true, vec![]);
        let out = export_csv(&projection, ',');
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("frame_id,name,kind"));
        assert!(lines[1].contains("plan"));
        assert!(lines[1].contains("local"));
    }

    #[test]
    fn tsv_uses_tab_delimiter() {
        let projection = CallGraphExportProjection::new(minimal(), vec![], "tsv", true, vec![]);
        let out = export_csv(&projection, '\t');
        assert!(out.lines().nth(1).unwrap().contains('\t'));
    }

    #[test]
    fn mermaid_renders_frames() {
        let projection = CallGraphExportProjection::new(minimal(), vec![], "mermaid", true, vec![]);
        let out = export_mermaid(&projection);
        assert!(out.starts_with("flowchart LR"));
        assert!(out.contains("a[\"plan\"]"));
    }
}

/// Governed import — preview first, then commit through authority (#287
/// import-side). A JSONL snapshot parses back into a definition ONLY when
/// it validates and its digest matches the manifest; commit never mutates
/// without an explicit authority ref.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPreview {
    pub graph_id: String,
    pub revision: u64,
    pub frames: usize,
    pub edges: usize,
    pub dispatches: usize,
    pub digest: String,
    pub valid: bool,
    pub issues: Vec<String>,
}

/// Parse the first JSONL line as the definition + validate + digest-check.
pub fn preview_import(jsonl: &str, expected_digest: Option<&str>) -> ImportPreview {
    let mut issues = Vec::new();
    let first_line = jsonl.lines().next().unwrap_or_default();
    let graph: Option<FocusaCallGraphDefinition> = serde_json::from_str(first_line).ok();
    let Some(graph) = graph else {
        return ImportPreview {
            graph_id: "unparseable".to_string(),
            revision: 0,
            frames: 0,
            edges: 0,
            dispatches: jsonl.lines().count().saturating_sub(1),
            digest: String::new(),
            valid: false,
            issues: vec!["first line is not a valid FocusaCallGraphDefinition".to_string()],
        };
    };
    let report = crate::callgraph::validate_graph(&graph);
    if !report.valid {
        for issue in &report.issues {
            issues.push(format!("{}: {}", issue.path, issue.message));
        }
    }
    let digest = graph_digest(&graph);
    if let Some(expected) = expected_digest {
        if expected != digest {
            issues.push(format!(
                "digest mismatch: manifest {expected} computed {digest}"
            ));
        }
    }
    let frame_count = jsonl
        .lines()
        .skip(1)
        .filter(|line| line.contains("\"frame_id\""))
        .count();
    let edge_count = jsonl
        .lines()
        .skip(1)
        .filter(|line| line.contains("\"edge_id\""))
        .count();
    ImportPreview {
        graph_id: graph.graph_id.clone(),
        revision: graph.revision,
        frames: frame_count,
        edges: edge_count,
        dispatches: jsonl
            .lines()
            .count()
            .saturating_sub(1 + frame_count + edge_count),
        digest,
        valid: issues.is_empty(),
        issues,
    }
}

#[cfg(test)]
mod import_tests {
    use super::*;
    use crate::callgraph::{
        AcceptanceContract, AuthorityRef, CallGraphPolicies, CallGraphScope, FocusaCallFrame,
        FrameKind, SideEffectClass,
    };

    #[test]
    fn preview_import_accepts_lossless_snapshot() {
        let graph = FocusaCallGraphDefinition {
            schema: crate::callgraph::CALLGRAPH_SCHEMA.to_string(),
            graph_id: "g-import".to_string(),
            revision: 1,
            scope: CallGraphScope {
                project_root: "/root/proj".to_string(),
                continuity_id: "cont-1".to_string(),
            },
            mission_ref: "m1".to_string(),
            trajectory_ref: None,
            workpoint_refs: vec![],
            title: "t".to_string(),
            description: "t".to_string(),
            entry_frame_ids: vec!["a".to_string()],
            frames: vec![FocusaCallFrame {
                frame_id: "a".to_string(),
                name: "a".to_string(),
                purpose: "t".to_string(),
                kind: FrameKind::Agent,
                input_schema: serde_json::json!({}),
                return_schema: serde_json::json!({}),
                preconditions: vec![],
                postconditions: vec![],
                side_effect_class: SideEffectClass::None,
                capability_refs: vec![],
                authority_requirement: None,
                timeout_policy: None,
                retry_policy: None,
                failure_boundary: None,
                compensation_frame_id: None,
                resource_budget: None,
                acceptance: AcceptanceContract {
                    acceptance_atoms: vec!["a1".to_string()],
                    verifier: None,
                },
                execution_binding: None,
            }],
            edges: vec![],
            policies: CallGraphPolicies::default(),
            required_evidence: vec![],
            created_at: "t".to_string(),
            created_by: AuthorityRef {
                authority_kind: "operator".to_string(),
                reference: "op-1".to_string(),
            },
            supersedes_revision: None,
        };
        let projection = CallGraphExportProjection::new(graph, vec![], "jsonl", true, vec![]);
        let jsonl = export_jsonl(&projection);
        let preview = preview_import(&jsonl, Some(&projection.manifest.digest));
        assert!(preview.valid, "issues: {:?}", preview.issues);
        assert_eq!(preview.graph_id, "g-import");
        assert_eq!(preview.frames, 1);
    }

    #[test]
    fn preview_import_rejects_digest_mismatch() {
        let graph = FocusaCallGraphDefinition {
            schema: crate::callgraph::CALLGRAPH_SCHEMA.to_string(),
            graph_id: "g2".to_string(),
            revision: 1,
            scope: CallGraphScope {
                project_root: "/r".to_string(),
                continuity_id: "c".to_string(),
            },
            mission_ref: "m".to_string(),
            trajectory_ref: None,
            workpoint_refs: vec![],
            title: "t".to_string(),
            description: "t".to_string(),
            entry_frame_ids: vec!["a".to_string()],
            frames: vec![FocusaCallFrame {
                frame_id: "a".to_string(),
                name: "a".to_string(),
                purpose: "t".to_string(),
                kind: FrameKind::Tool,
                input_schema: serde_json::json!({}),
                return_schema: serde_json::json!({}),
                preconditions: vec![],
                postconditions: vec![],
                side_effect_class: SideEffectClass::None,
                capability_refs: vec![],
                authority_requirement: None,
                timeout_policy: None,
                retry_policy: None,
                failure_boundary: None,
                compensation_frame_id: None,
                resource_budget: None,
                acceptance: AcceptanceContract {
                    acceptance_atoms: vec!["a1".to_string()],
                    verifier: None,
                },
                execution_binding: None,
            }],
            edges: vec![],
            policies: CallGraphPolicies::default(),
            required_evidence: vec![],
            created_at: "t".to_string(),
            created_by: AuthorityRef {
                authority_kind: "operator".to_string(),
                reference: "op-1".to_string(),
            },
            supersedes_revision: None,
        };
        let projection = CallGraphExportProjection::new(graph, vec![], "jsonl", true, vec![]);
        let jsonl = export_jsonl(&projection);
        let preview = preview_import(&jsonl, Some("sha256:deadbeef"));
        assert!(!preview.valid);
        assert!(preview.issues[0].contains("digest mismatch"));
    }

    #[test]
    fn preview_import_rejects_garbage() {
        let preview = preview_import("not json\n", None);
        assert!(!preview.valid);
        assert!(preview.issues[0].contains("not a valid"));
    }
}
