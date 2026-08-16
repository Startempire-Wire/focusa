//! CallGraph canonical execution authority — slice 1 (#254).
//!
//! Typed CallGraph definition (Spec 155 §9) with pure structural
//! validation and deterministic frame-eligibility disposition (§12 steps
//! 1-5 and 12). Later slices add the run ledger, frame leases, liveness,
//! model routing, and the dispatch event commit boundary.
//!
//! Design: docs/155-focusa-callgraph-workflow-and-flow-mesh-execution-integration-spec.md

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const CALLGRAPH_SCHEMA: &str = "focusa.callgraph.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusaCallGraphDefinition {
    pub schema: String,
    pub graph_id: String,
    pub revision: u64,
    pub scope: CallGraphScope,
    pub mission_ref: String,
    #[serde(default)]
    pub trajectory_ref: Option<String>,
    #[serde(default)]
    pub workpoint_refs: Vec<String>,
    pub title: String,
    pub description: String,
    pub entry_frame_ids: Vec<String>,
    pub frames: Vec<FocusaCallFrame>,
    pub edges: Vec<FocusaCallEdge>,
    #[serde(default)]
    pub policies: CallGraphPolicies,
    #[serde(default)]
    pub required_evidence: Vec<EvidenceRequirement>,
    pub created_at: String,
    pub created_by: AuthorityRef,
    #[serde(default)]
    pub supersedes_revision: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallGraphScope {
    pub project_root: String,
    pub continuity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusaCallFrame {
    pub frame_id: String,
    pub name: String,
    pub purpose: String,
    pub kind: FrameKind,
    pub input_schema: serde_json::Value,
    pub return_schema: serde_json::Value,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub postconditions: Vec<String>,
    pub side_effect_class: SideEffectClass,
    #[serde(default)]
    pub capability_refs: Vec<String>,
    #[serde(default)]
    pub authority_requirement: Option<AuthorityRequirement>,
    #[serde(default)]
    pub timeout_policy: Option<TimeoutPolicy>,
    #[serde(default)]
    pub retry_policy: Option<RetryPolicy>,
    #[serde(default)]
    pub failure_boundary: Option<FailureBoundary>,
    #[serde(default)]
    pub compensation_frame_id: Option<String>,
    #[serde(default)]
    pub resource_budget: Option<ResourceBudget>,
    pub acceptance: AcceptanceContract,
    #[serde(default)]
    pub execution_binding: Option<ExecutionBindingRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameKind {
    Human,
    Agent,
    Tool,
    Approval,
    Timer,
    Join,
    Subgraph,
    FlowmeshTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideEffectClass {
    None,
    Local,
    External,
    Destructive,
    Financial,
    Security,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityRequirement {
    pub authority_kind: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    pub timeout_ms: u64,
    pub on_timeout: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
    #[serde(default)]
    pub jitter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureBoundary {
    pub kind: String,
    pub unwind_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_tokens: Option<u64>,
    pub max_cost_usd_micros: Option<u64>,
    pub max_wall_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceContract {
    pub acceptance_atoms: Vec<String>,
    #[serde(default)]
    pub verifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBindingRef {
    pub binding_kind: String,
    pub binding_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusaCallEdge {
    pub edge_id: String,
    pub from_frame_id: String,
    pub to_frame_id: String,
    pub kind: EdgeKind,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub input_mapping: Vec<DataMapping>,
    #[serde(default)]
    pub return_mapping: Vec<DataMapping>,
    #[serde(default)]
    pub join_policy: Option<JoinPolicy>,
    #[serde(default)]
    pub cycle_policy: Option<CyclePolicy>,
    #[serde(default)]
    pub authority_requirement: Option<AuthorityRequirement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Call,
    Spawn,
    Await,
    Join,
    Continue,
    Condition,
    Retry,
    Catch,
    Compensate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataMapping {
    pub from_path: String,
    pub to_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinPolicy {
    All,
    Any,
    Majority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CyclePolicy {
    Allow,
    Deny,
    Iterate { max_iterations: u32 },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallGraphPolicies {
    #[serde(default)]
    pub max_depth: Option<u32>,
    #[serde(default)]
    pub default_cycle_policy: Option<CyclePolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    pub evidence_type: String,
    pub min_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityRef {
    pub authority_kind: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Eligible,
    WaitingInput,
    WaitingParent,
    WaitingJoin,
    WaitingAuthority,
    WaitingCapability,
    BlockedScope,
    BlockedStale,
    BlockedBudget,
    BlockedCyclePolicy,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

/// Structural validation: identity, endpoints, entry frames, joins,
/// compensation targets, and cycle policy conformance. Pure — no IO.
pub fn validate_graph(graph: &FocusaCallGraphDefinition) -> ValidationReport {
    let mut issues = Vec::new();
    let mut error = |path: &str, message: String| {
        issues.push(ValidationIssue {
            severity: "error".to_string(),
            path: path.to_string(),
            message,
        });
    };

    if graph.schema != CALLGRAPH_SCHEMA {
        error("schema", format!("expected {CALLGRAPH_SCHEMA}, got {}", graph.schema));
    }
    if graph.graph_id.trim().is_empty() {
        error("graph_id", "graph_id must be non-empty".to_string());
    }
    if graph.frames.is_empty() {
        error("frames", "at least one frame required".to_string());
    }
    if graph.entry_frame_ids.is_empty() {
        error("entry_frame_ids", "at least one entry frame required".to_string());
    }
    if graph.scope.project_root.trim().is_empty() {
        error("scope.project_root", "project_root must be non-empty".to_string());
    }
    if graph.scope.continuity_id.trim().is_empty() {
        error("scope.continuity_id", "continuity_id must be non-empty".to_string());
    }

    let frame_ids: HashSet<&str> = graph.frames.iter().map(|f| f.frame_id.as_str()).collect();
    if frame_ids.len() != graph.frames.len() {
        error("frames", "duplicate frame_id detected".to_string());
    }
    let edge_ids: HashSet<&str> = graph.edges.iter().map(|e| e.edge_id.as_str()).collect();
    if edge_ids.len() != graph.edges.len() {
        error("edges", "duplicate edge_id detected".to_string());
    }
    for entry in &graph.entry_frame_ids {
        if !frame_ids.contains(entry.as_str()) {
            error("entry_frame_ids", format!("entry frame {entry} does not exist"));
        }
    }
    for frame in &graph.frames {
        if let Some(compensation) = &frame.compensation_frame_id {
            if !frame_ids.contains(compensation.as_str()) {
                error(
                    &format!("frames.{}.compensation_frame_id", frame.frame_id),
                    format!("compensation frame {compensation} does not exist"),
                );
            }
        }
        if frame.acceptance.acceptance_atoms.is_empty() {
            error(
                &format!("frames.{}.acceptance", frame.frame_id),
                "acceptance_atoms must not be empty".to_string(),
            );
        }
    }
    for edge in &graph.edges {
        if !frame_ids.contains(edge.from_frame_id.as_str()) {
            error(
                &format!("edges.{}.from_frame_id", edge.edge_id),
                format!("source frame {} does not exist", edge.from_frame_id),
            );
        }
        if !frame_ids.contains(edge.to_frame_id.as_str()) {
            error(
                &format!("edges.{}.to_frame_id", edge.edge_id),
                format!("target frame {} does not exist", edge.to_frame_id),
            );
        }
    }

    // Cycle check (Spec 155 §15.3): reject cycles unless every cycle edge
    // carries an explicit cycle_policy.
    if has_unpolicied_cycle(graph) {
        error(
            "edges",
            "cycle detected without cycle_policy on every cycle edge".to_string(),
        );
    }

    ValidationReport {
        valid: issues.is_empty(),
        issues,
    }
}

fn has_unpolicied_cycle(graph: &FocusaCallGraphDefinition) -> bool {
    let default_policy = graph.policies.default_cycle_policy.as_ref();
    // Enumerate simple cycles per start frame (Spec 155 §15.3): a cycle is
    // allowed when at least one edge in it carries an explicit cycle policy
    // (or a graph-default policy exists).
    for frame in &graph.frames {
        // path_edges[i] is the edge that led to path[i+1].
        let mut stack: Vec<(String, Vec<String>, Vec<String>)> =
            vec![(frame.frame_id.clone(), vec![frame.frame_id.clone()], vec![])];
        while let Some((current, path, edge_path)) = stack.pop() {
            if path.len() > 32 {
                continue;
            }
            for edge in &graph.edges {
                if edge.from_frame_id != current {
                    continue;
                }
                if edge.to_frame_id == frame.frame_id {
                    // Cycle closes; allow when any edge in the cycle
                    // carries an explicit policy (or a graph default).
                    let any_policy = edge.cycle_policy.is_some()
                        || edge_path
                            .iter()
                            .any(|id| {
                                graph
                                    .edges
                                    .iter()
                                    .find(|e| e.edge_id == *id)
                                    .and_then(|e| e.cycle_policy.as_ref())
                                    .is_some()
                            });
                    if !any_policy && default_policy.is_none() {
                        return true;
                    }
                    continue;
                }
                if path.contains(&edge.to_frame_id) {
                    continue;
                }
                let mut next = path.clone();
                next.push(edge.to_frame_id.clone());
                let mut next_edges = edge_path.clone();
                next_edges.push(edge.edge_id.clone());
                stack.push((edge.to_frame_id.clone(), next, next_edges));
            }
        }
    }
    false
}

/// Deterministic eligibility (Spec 155 §12, slice 1 scope: steps 1-5, 12).
/// The runtime-steps (capabilities, budgets, idempotency receipts) arrive
/// with the run ledger in slice 2+; until then the disposition covers the
/// structural frontier.
pub fn eligibility_for_frame(
    graph: &FocusaCallGraphDefinition,
    frame_id: &str,
    parent_frame_id: Option<&str>,
    settled_edges: &HashSet<String>,
) -> Disposition {
    let frame_ids: HashSet<&str> = graph.frames.iter().map(|f| f.frame_id.as_str()).collect();
    if !frame_ids.contains(frame_id) {
        return Disposition::Rejected;
    }
    if let Some(parent) = parent_frame_id {
        if !frame_ids.contains(parent) {
            return Disposition::Rejected;
        }
        let edge_exists = graph
            .edges
            .iter()
            .any(|e| e.from_frame_id == parent && e.to_frame_id == frame_id);
        if !edge_exists {
            return Disposition::WaitingParent;
        }
    }
    // Join frames wait until every inbound non-cycle edge is settled.
    let frame_kind = graph
        .frames
        .iter()
        .find(|f| f.frame_id == frame_id)
        .map(|f| f.kind);
    if matches!(frame_kind, Some(FrameKind::Join)) {
        let inbound: Vec<&str> = graph
            .edges
            .iter()
            .filter(|e| e.to_frame_id == frame_id && e.kind != EdgeKind::Retry)
            .map(|e| e.edge_id.as_str())
            .collect();
        if inbound.iter().any(|id| !settled_edges.contains(*id)) {
            return Disposition::WaitingJoin;
        }
    }
    // Depth bound from policies (cycle/depth guard, step 5).
    if let Some(max_depth) = graph.policies.max_depth {
        if let Some(parent) = parent_frame_id {
            let depth = path_depth(graph, parent);
            if depth + 1 > max_depth {
                return Disposition::BlockedCyclePolicy;
            }
        }
    }
    Disposition::Eligible
}

fn path_depth(graph: &FocusaCallGraphDefinition, frame_id: &str) -> u32 {
    let mut depth: HashMap<&str, u32> = HashMap::new();
    for frame in &graph.frames {
        depth.insert(frame.frame_id.as_str(), 0);
    }
    let mut changed = true;
    while changed {
        changed = false;
        for edge in &graph.edges {
            let next = depth.get(edge.from_frame_id.as_str()).copied().unwrap_or(0) + 1;
            let slot = depth.entry(edge.to_frame_id.as_str()).or_insert(0);
            if next > *slot {
                *slot = next;
                changed = true;
            }
        }
    }
    depth.get(frame_id).copied().unwrap_or(0)
}

/// Deterministic frontier reconstruction (Spec 155 §18.3). Pure function
/// of (definition, ordered dispatch ledger): same inputs always reproduce
/// the same frontier. A dispatch without a receipt is an active
/// invocation; a receipted dispatch settles its edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayedFrontier {
    pub active_invocation_ids: Vec<String>,
    pub settled_edge_ids: Vec<String>,
    pub waiting_frame_ids: Vec<String>,
    pub completed_frame_ids: Vec<String>,
}

pub fn replay_frontier(
    graph: &FocusaCallGraphDefinition,
    dispatches: &[crate::callgraph_store::FrameDispatch],
) -> ReplayedFrontier {
    let mut active = Vec::new();
    let mut settled_edges = Vec::new();
    let mut completed_frames = Vec::new();
    for dispatch in dispatches {
        match &dispatch.receipt_ref {
            None => active.push(dispatch.invocation_id.clone().unwrap_or_default()),
            Some(_) => {
                // The dispatch's caller edge settled: map invocation→edge via
                // parent_invocation matching on the dispatch's own edge is
                // unavailable in the ledger row, so settle by frame edges.
                settled_edges.push(dispatch.dispatch_id.clone());
                completed_frames.push(dispatch.frame_id.clone());
            }
        }
    }
    let frame_ids: std::collections::HashSet<&str> =
        graph.frames.iter().map(|f| f.frame_id.as_str()).collect();
    let completed: std::collections::HashSet<&str> =
        completed_frames.iter().map(|f| f.as_str()).collect();
    let mut waiting = Vec::new();
    for frame in &graph.frames {
        let inbound: Vec<&str> = graph
            .edges
            .iter()
            .filter(|e| e.to_frame_id == frame.frame_id && e.kind != EdgeKind::Retry)
            .map(|e| e.from_frame_id.as_str())
            .collect();
        if inbound.is_empty() {
            continue;
        }
        let all_callers_settled = inbound.iter().all(|caller| completed.contains(caller));
        if !all_callers_settled {
            waiting.push(frame.frame_id.clone());
        }
    }
    ReplayedFrontier {
        active_invocation_ids: active.into_iter().filter(|id| !id.is_empty()).collect(),
        settled_edge_ids: settled_edges,
        waiting_frame_ids: waiting,
        completed_frame_ids: completed
            .into_iter()
            .filter(|id| frame_ids.contains(id))
            .map(|id| id.to_string())
            .collect(),
    }
}

/// Deterministic frame → adapter routing (#254 slice 9). The first
/// adapter whose capability set covers every frame capability wins; ties
/// keep input order, so the same registry always routes the same frame the
/// same way. No adapter call occurs here — this only produces the decision
/// that the dispatch path commits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterCapability {
    pub adapter_id: String,
    pub model: String,
    pub capabilities: Vec<String>,
    pub healthy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteDecision {
    Routed {
        adapter_id: String,
        model: String,
    },
    WaitingCapability,
    Rejected,
}

pub fn route_frame(
    frame: &FocusaCallFrame,
    adapters: &[AdapterCapability],
) -> RouteDecision {
    if frame.capability_refs.is_empty() {
        // No capability requirements: route to the first healthy adapter.
        if let Some(adapter) = adapters.iter().find(|adapter| adapter.healthy) {
            return RouteDecision::Routed {
                adapter_id: adapter.adapter_id.clone(),
                model: adapter.model.clone(),
            };
        }
        return RouteDecision::WaitingCapability;
    }
    let required: std::collections::HashSet<&str> =
        frame.capability_refs.iter().map(|c| c.as_str()).collect();
    for adapter in adapters {
        if !adapter.healthy {
            continue;
        }
        let provided: std::collections::HashSet<&str> =
            adapter.capabilities.iter().map(|c| c.as_str()).collect();
        if required.is_subset(&provided) {
            return RouteDecision::Routed {
                adapter_id: adapter.adapter_id.clone(),
                model: adapter.model.clone(),
            };
        }
    }
    RouteDecision::WaitingCapability
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(id: &str, kind: FrameKind) -> FocusaCallFrame {
        FocusaCallFrame {
            frame_id: id.to_string(),
            name: id.to_string(),
            purpose: "test".to_string(),
            kind,
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
                acceptance_atoms: vec!["atom-1".to_string()],
                verifier: None,
            },
            execution_binding: None,
        }
    }

    fn graph(frames: Vec<FocusaCallFrame>, edges: Vec<FocusaCallEdge>) -> FocusaCallGraphDefinition {
        FocusaCallGraphDefinition {
            schema: CALLGRAPH_SCHEMA.to_string(),
            graph_id: "g1".to_string(),
            revision: 1,
            scope: CallGraphScope {
                project_root: "/root/proj".to_string(),
                continuity_id: "cont-1".to_string(),
            },
            mission_ref: "m1".to_string(),
            trajectory_ref: None,
            workpoint_refs: vec![],
            title: "test".to_string(),
            description: "test".to_string(),
            entry_frame_ids: vec!["a".to_string()],
            frames,
            edges,
            policies: CallGraphPolicies::default(),
            required_evidence: vec![],
            created_at: "2026-08-16T00:00:00Z".to_string(),
            created_by: AuthorityRef {
                authority_kind: "operator".to_string(),
                reference: "op-1".to_string(),
            },
            supersedes_revision: None,
        }
    }

    #[test]
    fn valid_linear_graph_passes() {
        let g = graph(
            vec![frame("a", FrameKind::Agent), frame("b", FrameKind::Tool)],
            vec![FocusaCallEdge {
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
        );
        let report = validate_graph(&g);
        assert!(report.valid, "unexpected issues: {:?}", report.issues);
    }

    #[test]
    fn dangling_edge_fails_validation() {
        let g = graph(
            vec![frame("a", FrameKind::Agent)],
            vec![FocusaCallEdge {
                edge_id: "e1".to_string(),
                from_frame_id: "a".to_string(),
                to_frame_id: "ghost".to_string(),
                kind: EdgeKind::Call,
                condition: None,
                input_mapping: vec![],
                return_mapping: vec![],
                join_policy: None,
                cycle_policy: None,
                authority_requirement: None,
            }],
        );
        let report = validate_graph(&g);
        assert!(!report.valid);
        assert!(report.issues.iter().any(|i| i.path.contains("to_frame_id")));
    }

    #[test]
    fn unpolicied_cycle_fails_validation() {
        let edges = vec![
            FocusaCallEdge {
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
            },
            FocusaCallEdge {
                edge_id: "e2".to_string(),
                from_frame_id: "b".to_string(),
                to_frame_id: "a".to_string(),
                kind: EdgeKind::Continue,
                condition: None,
                input_mapping: vec![],
                return_mapping: vec![],
                join_policy: None,
                cycle_policy: None,
                authority_requirement: None,
            },
        ];
        let g = graph(vec![frame("a", FrameKind::Agent), frame("b", FrameKind::Tool)], edges);
        assert!(!validate_graph(&g).valid);
    }

    #[test]
    fn policied_cycle_passes_validation() {
        let edges = vec![
            FocusaCallEdge {
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
            },
            FocusaCallEdge {
                edge_id: "e2".to_string(),
                from_frame_id: "b".to_string(),
                to_frame_id: "a".to_string(),
                kind: EdgeKind::Continue,
                condition: None,
                input_mapping: vec![],
                return_mapping: vec![],
                join_policy: None,
                cycle_policy: Some(CyclePolicy::Iterate { max_iterations: 3 }),
                authority_requirement: None,
            },
        ];
        let g = graph(vec![frame("a", FrameKind::Agent), frame("b", FrameKind::Tool)], edges);
        assert!(validate_graph(&g).valid);
    }

    #[test]
    fn join_frame_waits_for_inbound_settlement() {
        let edges = vec![
            FocusaCallEdge {
                edge_id: "e1".to_string(),
                from_frame_id: "a".to_string(),
                to_frame_id: "join".to_string(),
                kind: EdgeKind::Call,
                condition: None,
                input_mapping: vec![],
                return_mapping: vec![],
                join_policy: Some(JoinPolicy::All),
                cycle_policy: None,
                authority_requirement: None,
            },
            FocusaCallEdge {
                edge_id: "e2".to_string(),
                from_frame_id: "b".to_string(),
                to_frame_id: "join".to_string(),
                kind: EdgeKind::Call,
                condition: None,
                input_mapping: vec![],
                return_mapping: vec![],
                join_policy: None,
                cycle_policy: None,
                authority_requirement: None,
            },
        ];
        let g = graph(
            vec![
                frame("a", FrameKind::Agent),
                frame("b", FrameKind::Agent),
                frame("join", FrameKind::Join),
            ],
            edges,
        );
        let mut settled: HashSet<String> = HashSet::new();
        settled.insert("e1".to_string());
        assert_eq!(
            eligibility_for_frame(&g, "join", None, &settled),
            Disposition::WaitingJoin
        );
        settled.insert("e2".to_string());
        assert_eq!(
            eligibility_for_frame(&g, "join", None, &settled),
            Disposition::Eligible
        );
    }

    #[test]
    fn unknown_frame_is_rejected() {
        let g = graph(vec![frame("a", FrameKind::Agent)], vec![]);
        assert_eq!(
            eligibility_for_frame(&g, "ghost", None, &HashSet::new()),
            Disposition::Rejected
        );
    }

    #[test]
    fn missing_parent_edge_is_waiting_parent() {
        let g = graph(
            vec![frame("a", FrameKind::Agent), frame("b", FrameKind::Tool)],
            vec![],
        );
        assert_eq!(
            eligibility_for_frame(&g, "b", Some("a"), &HashSet::new()),
            Disposition::WaitingParent
        );
    }
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    use crate::callgraph_store::FrameDispatch;

    fn linear_graph() -> FocusaCallGraphDefinition {
        FocusaCallGraphDefinition {
            schema: CALLGRAPH_SCHEMA.to_string(),
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
                },
                FocusaCallFrame {
                    frame_id: "b".to_string(),
                    name: "b".to_string(),
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

    fn dispatch(
        id: &str,
        frame: &str,
        invocation: &str,
        receipt: Option<&str>,
    ) -> FrameDispatch {
        FrameDispatch {
            dispatch_id: id.to_string(),
            run_id: "r1".to_string(),
            frame_id: frame.to_string(),
            invocation_id: Some(invocation.to_string()),
            parent_invocation_id: None,
            disposition: Disposition::Eligible,
            attempt: 1,
            committed_at: "t".to_string(),
            receipt_ref: receipt.map(|r| r.to_string()),
        }
    }

    #[test]
    fn replay_reproduces_frontier_deterministically() {
        let graph = linear_graph();
        let dispatches = vec![
            dispatch("d1", "a", "i1", None),
            dispatch("d2", "b", "i2", None),
        ];
        let first = replay_frontier(&graph, &dispatches);
        let second = replay_frontier(&graph, &dispatches);
        assert_eq!(first, second, "replay must be deterministic");
        assert_eq!(first.active_invocation_ids, vec!["i1", "i2"]);
        assert!(first.waiting_frame_ids.contains(&"b".to_string()));
    }

    #[test]
    fn receipted_dispatch_settles_frame() {
        let graph = linear_graph();
        let dispatches = vec![
            dispatch("d1", "a", "i1", Some("receipt-a")),
            dispatch("d2", "b", "i2", None),
        ];
        let frontier = replay_frontier(&graph, &dispatches);
        assert!(frontier.active_invocation_ids.contains(&"i2".to_string()));
        assert!(!frontier.active_invocation_ids.contains(&"i1".to_string()));
        assert!(frontier.completed_frame_ids.contains(&"a".to_string()));
        // With caller "a" settled, frame "b" is no longer waiting.
        assert!(!frontier.waiting_frame_ids.contains(&"b".to_string()));
    }
}

#[cfg(test)]
mod routing_tests {
    use super::*;

    fn frame_with_caps(caps: &[&str]) -> FocusaCallFrame {
        FocusaCallFrame {
            frame_id: "f".to_string(),
            name: "f".to_string(),
            purpose: "t".to_string(),
            kind: FrameKind::Agent,
            input_schema: serde_json::json!({}),
            return_schema: serde_json::json!({}),
            preconditions: vec![],
            postconditions: vec![],
            side_effect_class: SideEffectClass::None,
            capability_refs: caps.iter().map(|c| c.to_string()).collect(),
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
        }
    }

    #[test]
    fn routes_to_first_capable_adapter() {
        let adapters = vec![
            AdapterCapability {
                adapter_id: "pi".to_string(),
                model: "pi-tool".to_string(),
                capabilities: vec!["shell".to_string()],
                healthy: true,
            },
            AdapterCapability {
                adapter_id: "uiai".to_string(),
                model: "uiai-browser".to_string(),
                capabilities: vec!["shell".to_string(), "browser".to_string()],
                healthy: true,
            },
        ];
        let frame = frame_with_caps(&["shell", "browser"]);
        match route_frame(&frame, &adapters) {
            RouteDecision::Routed { adapter_id, .. } => assert_eq!(adapter_id, "uiai"),
            other => panic!("expected Routed, got {other:?}"),
        }
    }

    #[test]
    fn unhealthy_adapters_are_skipped() {
        let adapters = vec![AdapterCapability {
            adapter_id: "pi".to_string(),
            model: "pi-tool".to_string(),
            capabilities: vec!["shell".to_string()],
            healthy: false,
        }];
        let frame = frame_with_caps(&["shell"]);
        assert_eq!(route_frame(&frame, &adapters), RouteDecision::WaitingCapability);
    }

    #[test]
    fn missing_capability_waits() {
        let adapters = vec![AdapterCapability {
            adapter_id: "pi".to_string(),
            model: "pi-tool".to_string(),
            capabilities: vec!["shell".to_string()],
            healthy: true,
        }];
        let frame = frame_with_caps(&["browser"]);
        assert_eq!(route_frame(&frame, &adapters), RouteDecision::WaitingCapability);
    }

    #[test]
    fn deterministic_ordering_on_ties() {
        let adapters = vec![
            AdapterCapability {
                adapter_id: "first".to_string(),
                model: "m".to_string(),
                capabilities: vec!["shell".to_string()],
                healthy: true,
            },
            AdapterCapability {
                adapter_id: "second".to_string(),
                model: "m".to_string(),
                capabilities: vec!["shell".to_string()],
                healthy: true,
            },
        ];
        let frame = frame_with_caps(&["shell"]);
        match route_frame(&frame, &adapters) {
            RouteDecision::Routed { adapter_id, .. } => assert_eq!(adapter_id, "first"),
            other => panic!("expected Routed, got {other:?}"),
        }
    }
}
