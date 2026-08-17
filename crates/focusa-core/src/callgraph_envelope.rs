//! CallGraph Item Envelope — slice 1 (#289).
//!
//! One canonical, layered, self-describing envelope for every CallGraph
//! item. Humans and machines consume the same structure: identity,
//! semantic meaning, topology, execution, assignment, authority,
//! verification, completion, and provenance layers — each typed, bounded,
//! and digest-bearing. No consumer reconstructs meaning from labels or
//! titles (core invariant).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::callgraph::{
    FocusaCallFrame, FocusaCallGraphDefinition, FrameKind,
};

pub const ITEM_ENVELOPE_SCHEMA: &str = "focusa.callgraph_item_envelope.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemEnvelope {
    pub schema: String,
    pub schema_version: String,
    pub scope: crate::callgraph::CallGraphScope,
    pub workstream_id: String,
    pub graph_ref: GraphRef,
    pub frame_ref: FrameRef,
    pub identity: IdentityLayer,
    pub semantic: SemanticLayer,
    pub topology: TopologyLayer,
    pub execution: ExecutionLayer,
    pub assignment: AssignmentLayer,
    pub authority: AuthorityLayer,
    pub verification: VerificationLayer,
    pub completion: CompletionLayer,
    pub provenance: ProvenanceLayer,
    pub content_digest: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphRef {
    pub graph_id: String,
    pub revision: u64,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameRef {
    pub frame_id: String,
    pub kind: FrameKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityLayer {
    pub canonical_ref: String,
    pub display_id: String,
    pub title: String,
    pub aliases: Vec<String>,
    pub item_kind: String,
    pub parent_refs: Vec<String>,
    pub subject_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticLayer {
    pub purpose: String,
    pub action_type: String,
    pub target_refs: Vec<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub done_condition: String,
    pub not_done_if: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyLayer {
    pub entrypoint: bool,
    pub topological_rank: u32,
    pub depth: u32,
    pub caller_refs: Vec<String>,
    pub callee_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLayer {
    pub side_effect_class: crate::callgraph::SideEffectClass,
    pub capability_refs: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub retry_max_attempts: Option<u32>,
    pub resource_budget: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentLayer {
    pub assigned: bool,
    pub adapter_id: Option<String>,
    pub model: Option<String>,
    pub assignment_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityLayer {
    pub authority_requirement: Option<crate::callgraph::AuthorityRequirement>,
    pub mutation_confirmation_required: bool,
    pub authority_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationLayer {
    pub acceptance_atoms: Vec<String>,
    pub verifier: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionLayer {
    pub completed: bool,
    pub settled_at: Option<String>,
    pub receipt_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceLayer {
    pub source_spec_refs: Vec<String>,
    pub created_by: crate::callgraph::AuthorityRef,
    pub created_at: String,
}

/// Build the canonical envelope for one frame. Explicit by construction:
/// missing values stay explicit (None / empty), never guessed.
pub fn build_item_envelope(
    graph: &FocusaCallGraphDefinition,
    frame: &FocusaCallFrame,
    assigned: Option<(String, String)>,
) -> ItemEnvelope {
    let now = chrono::Utc::now().to_rfc3339();
    let graph_digest = crate::callgraph_export::graph_digest(graph);
    let workstream_id = crate::workstream_root::workstream_scope_key(
        &graph.scope.project_root,
        &graph.scope.continuity_id,
    );
    let caller_refs: Vec<String> = graph
        .edges
        .iter()
        .filter(|edge| edge.to_frame_id == frame.frame_id)
        .map(|edge| edge.from_frame_id.clone())
        .collect();
    let callee_refs: Vec<String> = graph
        .edges
        .iter()
        .filter(|edge| edge.from_frame_id == frame.frame_id)
        .map(|edge| edge.to_frame_id.clone())
        .collect();
    let canonical_ref = format!(
        "{}:{}:{}",
        graph.graph_id, graph.revision, frame.frame_id
    );
    let assigned = assigned.as_ref();
    let adapter_id = assigned.map(|pair| pair.0.clone());
    let model = assigned.map(|pair| pair.1.clone());
    let mut envelope = ItemEnvelope {
        schema: ITEM_ENVELOPE_SCHEMA.to_string(),
        schema_version: "1".to_string(),
        scope: graph.scope.clone(),
        workstream_id,
        graph_ref: GraphRef {
            graph_id: graph.graph_id.clone(),
            revision: graph.revision,
            digest: graph_digest,
        },
        frame_ref: FrameRef {
            frame_id: frame.frame_id.clone(),
            kind: frame.kind,
        },
        identity: IdentityLayer {
            canonical_ref,
            display_id: frame.frame_id.clone(),
            title: frame.name.clone(),
            aliases: vec![],
            item_kind: match frame.kind {
                FrameKind::Human => "human_task",
                FrameKind::Agent => "agent_task",
                FrameKind::Tool => "tool_invocation",
                FrameKind::Approval => "approval_gate",
                FrameKind::Timer => "timer",
                FrameKind::Join => "join_barrier",
                FrameKind::Subgraph => "subgraph",
                FrameKind::FlowmeshTask => "flowmesh_task",
            }
            .to_string(),
            parent_refs: graph
                .workpoint_refs
                .iter()
                .map(|wp| format!("workpoint:{wp}"))
                .collect(),
            subject_refs: vec![graph.mission_ref.clone()],
        },
        semantic: SemanticLayer {
            purpose: frame.purpose.clone(),
            action_type: format!("{:?}", frame.kind).to_lowercase(),
            target_refs: frame.capability_refs.clone(),
            input_schema: frame.input_schema.clone(),
            output_schema: frame.return_schema.clone(),
            preconditions: frame.preconditions.clone(),
            postconditions: frame.postconditions.clone(),
            done_condition: frame.acceptance.acceptance_atoms.join(" AND "),
            not_done_if: vec![],
        },
        topology: TopologyLayer {
            entrypoint: graph.entry_frame_ids.contains(&frame.frame_id),
            topological_rank: caller_refs.len() as u32,
            depth: path_depth_of(graph, &frame.frame_id),
            caller_refs,
            callee_refs,
        },
        execution: ExecutionLayer {
            side_effect_class: frame.side_effect_class,
            capability_refs: frame.capability_refs.clone(),
            timeout_ms: frame.timeout_policy.as_ref().map(|policy| policy.timeout_ms),
            retry_max_attempts: frame.retry_policy.as_ref().map(|policy| policy.max_attempts),
            resource_budget: frame
                .resource_budget
                .as_ref()
                .and_then(|budget| serde_json::to_value(budget).ok()),
        },
        assignment: AssignmentLayer {
            assigned: adapter_id.is_some(),
            adapter_id,
            model,
            assignment_source: if assigned.is_some() {
                "callgraph_route".to_string()
            } else {
                "unassigned".to_string()
            },
        },
        authority: AuthorityLayer {
            authority_requirement: frame.authority_requirement.clone(),
            mutation_confirmation_required: matches!(
                frame.side_effect_class,
                crate::callgraph::SideEffectClass::Destructive
                    | crate::callgraph::SideEffectClass::Financial
                    | crate::callgraph::SideEffectClass::Security
            ),
            authority_refs: vec![format!(
                "created_by:{}:{}",
                graph.created_by.authority_kind, graph.created_by.reference
            )],
        },
        verification: VerificationLayer {
            acceptance_atoms: frame.acceptance.acceptance_atoms.clone(),
            verifier: frame.acceptance.verifier.clone(),
            evidence_refs: vec![],
        },
        completion: CompletionLayer {
            completed: false,
            settled_at: None,
            receipt_ref: None,
        },
        provenance: ProvenanceLayer {
            source_spec_refs: vec!["docs/155-focusa-callgraph-workflow-and-flow-mesh-execution-integration-spec.md".to_string()],
            created_by: graph.created_by.clone(),
            created_at: graph.created_at.clone(),
        },
        content_digest: String::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    envelope.content_digest = envelope_digest(&envelope);
    envelope
}

fn path_depth_of(graph: &FocusaCallGraphDefinition, frame_id: &str) -> u32 {
    // Bounded depth via caller walk.
    let mut depth = 0u32;
    let mut frontier: Vec<&str> = vec![frame_id];
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    while let Some(current) = frontier.pop() {
        if !seen.insert(current) {
            continue;
        }
        let callers: Vec<&str> = graph
            .edges
            .iter()
            .filter(|edge| edge.to_frame_id == current)
            .map(|edge| edge.from_frame_id.as_str())
            .collect();
        if !callers.is_empty() {
            depth += 1;
        }
        for caller in callers {
            frontier.push(caller);
        }
    }
    depth
}

fn envelope_digest(envelope: &ItemEnvelope) -> String {
    let mut hasher = Sha256::new();
    let identity = format!(
        "{}|{}|{}|{}",
        envelope.identity.canonical_ref,
        envelope.identity.title,
        envelope.identity.item_kind,
        envelope.frame_ref.frame_id
    );
    hasher.update(identity.as_bytes());
    for atom in &envelope.verification.acceptance_atoms {
        hasher.update(atom.as_bytes());
    }
    hasher.update(format!("{:?}", envelope.execution.side_effect_class).as_bytes());
    format!("sha256:{}", hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::{
        AcceptanceContract, AuthorityRef, CallGraphPolicies, CallGraphScope, EdgeKind,
        FocusaCallEdge, SideEffectClass,
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
            workpoint_refs: vec!["wp1".to_string()],
            title: "t".to_string(),
            description: "t".to_string(),
            entry_frame_ids: vec!["a".to_string()],
            frames: vec![
                FocusaCallFrame {
                    frame_id: "a".to_string(),
                    name: "plan".to_string(),
                    purpose: "produce the plan".to_string(),
                    kind: FrameKind::Agent,
                    input_schema: serde_json::json!({}),
                    return_schema: serde_json::json!({}),
                    preconditions: vec![],
                    postconditions: vec![],
                    side_effect_class: SideEffectClass::Security,
                    capability_refs: vec!["shell".to_string()],
                    authority_requirement: None,
                    timeout_policy: None,
                    retry_policy: None,
                    failure_boundary: None,
                    compensation_frame_id: None,
                    resource_budget: None,
                    acceptance: AcceptanceContract {
                        acceptance_atoms: vec!["a1".to_string(), "a2".to_string()],
                        verifier: Some("operator".to_string()),
                    },
                    execution_binding: None,
                },
                FocusaCallFrame {
                    frame_id: "b".to_string(),
                    name: "approve".to_string(),
                    purpose: "approve the plan".to_string(),
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
    fn envelope_is_layered_and_explicit() {
        let graph = sample();
        let frame = graph.frames[0].clone();
        let envelope = build_item_envelope(&graph, &frame, Some(("pi".to_string(), "m".to_string())));
        assert_eq!(envelope.schema, ITEM_ENVELOPE_SCHEMA);
        assert_eq!(envelope.identity.canonical_ref, "g1:1:a");
        assert_eq!(envelope.identity.item_kind, "agent_task");
        assert!(envelope.topology.entrypoint);
        assert!(envelope.topology.callee_refs.contains(&"b".to_string()));
        assert!(envelope.authority.mutation_confirmation_required); // Security class
        assert_eq!(envelope.verification.acceptance_atoms.len(), 2);
        assert_eq!(envelope.assignment.adapter_id.as_deref(), Some("pi"));
        assert!(!envelope.content_digest.is_empty());
        assert!(!envelope.workstream_id.is_empty());
    }

    #[test]
    fn envelope_digest_is_stable_for_same_inputs() {
        let graph = sample();
        let frame = graph.frames[1].clone();
        let e1 = build_item_envelope(&graph, &frame, None);
        let e2 = build_item_envelope(&graph, &frame, None);
        assert_eq!(e1.content_digest, e2.content_digest);
    }

    #[test]
    fn envelope_serializes_as_machine_readable_json() {
        let graph = sample();
        let frame = graph.frames[0].clone();
        let envelope = build_item_envelope(&graph, &frame, None);
        let value = serde_json::to_value(&envelope).unwrap();
        assert!(value.get("identity").is_some());
        assert!(value.get("semantic").is_some());
        assert!(value.get("completion").is_some());
        assert!(value.get("provenance").is_some());
    }
}
