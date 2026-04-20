//! PRE (Proposal Resolution Engine) routes.

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use focusa_core::reducer;
use focusa_core::types::{
    Action, EventLogEntry, FocusaEvent, ProposalKind, ProposalStatus, SignalOrigin,
};
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

/// GET /v1/proposals — list pending proposals.
async fn list_proposals(State(state): State<Arc<AppState>>) -> Json<Value> {
    let s = state.focusa.read().await;
    Json(json!({
        "proposals": s.pre.proposals,
        "pending": focusa_core::pre::pending_count(&s.pre),
    }))
}

/// POST /v1/proposals — submit a proposal via daemon command channel.
async fn submit_proposal(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let kind_str = body
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("focus_change");
    let source = body.get("source").and_then(|v| v.as_str()).unwrap_or("api");
    let payload = body.get("payload").cloned().unwrap_or(json!({}));
    let deadline_ms = body
        .get("deadline_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(5000);
    let score = body.get("score").and_then(|v| v.as_f64());

    let kind = parse_proposal_kind(kind_str);

    let payload_for_audit = payload.clone();
    let submit_source = source.to_string();

    let _ = state
        .command_tx
        .send(Action::SubmitProposal {
            kind,
            source: submit_source.clone(),
            payload,
            deadline_ms,
            score,
        })
        .await;

    let mut proposal_id: Option<Uuid> = None;
    for _ in 0..240 {
        {
            let s = state.focusa.read().await;
            proposal_id = s
                .pre
                .proposals
                .iter()
                .rev()
                .find(|p| p.kind == kind && p.source == submit_source)
                .map(|p| p.id);
        }
        if proposal_id.is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    if let Ok(machine_id) = state.persistence.machine_id() {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: chrono::Utc::now(),
            event: submission_audit_event(
                proposal_id.unwrap_or_else(Uuid::now_v7),
                kind,
                source,
                &payload_for_audit,
            ),
            correlation_id: Some("api:submit_proposal".to_string()),
            origin: SignalOrigin::Cli,
            machine_id: Some(machine_id),
            instance_id: None,
            session_id: None,
            thread_id: None,
            is_observation: false,
        };
        let _ = state.persistence.append_event(&entry);
    }

    Json(json!({
        "status": "accepted",
        "proposal_id": proposal_id.map(|id| id.to_string()),
        "kind": kind_str,
        "target_class": proposal_target_class(kind),
    }))
}

fn apply_focus_change_proposal(
    state: focusa_core::types::FocusaState,
    winner: &focusa_core::types::Proposal,
    machine_id: &str,
) -> Result<focusa_core::types::ReductionResult, String> {
    let title = winner
        .payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("proposal-focus-change")
        .to_string();
    let goal = winner
        .payload
        .get("goal")
        .and_then(|v| v.as_str())
        .unwrap_or(&title)
        .to_string();
    let beads_issue_id = winner
        .payload
        .get("beads_issue_id")
        .and_then(|v| v.as_str())
        .unwrap_or("proposal-focus-change")
        .to_string();
    let constraints = winner
        .payload
        .get("constraints")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let tags = winner
        .payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    reducer::reduce_with_meta(
        state,
        FocusaEvent::FocusFramePushed {
            frame_id: Uuid::now_v7(),
            beads_issue_id,
            title,
            goal,
            constraints,
            tags,
        },
        Some(machine_id),
        None,
        false,
    )
    .map_err(|e| e.to_string())
}

fn thesis_update_event(winner: &focusa_core::types::Proposal) -> Result<FocusaEvent, String> {
    let primary_intent = winner
        .payload
        .get("primary_intent")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "thesis_update requires payload.primary_intent".to_string())?
        .to_string();

    let secondary_goals = winner
        .payload
        .get("secondary_goals")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let open_questions = winner
        .payload
        .get("open_questions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let assumptions = winner
        .payload
        .get("assumptions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let sources = winner
        .payload
        .get("sources")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let confidence_score = winner
        .payload
        .get("confidence")
        .and_then(|v| v.get("score"))
        .and_then(|v| v.as_f64())
        .unwrap_or(winner.score);
    let confidence_rationale = winner
        .payload
        .get("confidence")
        .and_then(|v| v.get("rationale"))
        .and_then(|v| v.as_str())
        .unwrap_or("proposal resolution accepted")
        .to_string();

    let thread_id = winner
        .payload
        .get("thread_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| "thesis_update requires payload.thread_id".to_string())?;

    Ok(FocusaEvent::ThreadThesisUpdated {
        thread_id,
        thesis: focusa_core::types::ThreadThesis {
            primary_intent,
            secondary_goals,
            constraints: focusa_core::types::ThesisConstraints::default(),
            open_questions,
            assumptions,
            confidence: focusa_core::types::ThesisConfidence {
                score: confidence_score,
                rationale: confidence_rationale,
            },
            scope: focusa_core::types::ThesisScope::default(),
            sources,
            updated_at: Some(chrono::Utc::now()),
        },
    })
}

fn memory_write_event(winner: &focusa_core::types::Proposal) -> Result<FocusaEvent, String> {
    let key = winner
        .payload
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "memory_write requires payload.key".to_string())?
        .to_string();
    let value = winner
        .payload
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "memory_write requires payload.value".to_string())?
        .to_string();
    let source = winner
        .payload
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("proposal");

    Ok(FocusaEvent::SemanticMemoryUpserted {
        key,
        value,
        source: source.to_string(),
    })
}

fn autonomy_adjustment_event(winner: &focusa_core::types::Proposal) -> Result<FocusaEvent, String> {
    let level = winner
        .payload
        .get("level")
        .and_then(|v| v.as_str())
        .and_then(parse_autonomy_level)
        .ok_or_else(|| "autonomy_adjustment requires payload.level (AL0-AL5)".to_string())?;
    let scope = winner
        .payload
        .get("scope")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let ttl_seconds = winner.payload.get("ttl_seconds").and_then(|v| v.as_i64());
    let ttl = ttl_seconds.map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs));
    let reason = winner
        .payload
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("proposal resolution accepted");

    Ok(FocusaEvent::AutonomyAdjusted {
        level,
        scope,
        ttl,
        reason: reason.to_string(),
    })
}

fn constitution_revision_event(
    winner: &focusa_core::types::Proposal,
) -> Result<FocusaEvent, String> {
    let version = winner
        .payload
        .get("version")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("proposal-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S")));
    let agent_id = winner
        .payload
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("wirebot");
    let principles = winner
        .payload
        .get("principles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    v.as_str()
                        .map(|text| focusa_core::types::ConstitutionPrinciple {
                            id: format!("p{}", i + 1),
                            text: text.to_string(),
                            priority: (i + 1) as u32,
                            rationale: String::new(),
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let safety_rules: Vec<String> = winner
        .payload
        .get("safety_rules")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let expression_rules: Vec<String> = winner
        .payload
        .get("expression_rules")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if principles.is_empty() && safety_rules.is_empty() && expression_rules.is_empty() {
        return Err(
            "constitution_revision requires at least one principle/safety/expression rule"
                .to_string(),
        );
    }

    Ok(FocusaEvent::ConstitutionLoaded {
        version,
        agent_id: agent_id.to_string(),
        principles,
        safety_rules,
        expression_rules,
    })
}

fn parse_proposal_kind(kind_str: &str) -> ProposalKind {
    match kind_str {
        "focus_change" => ProposalKind::FocusChange,
        "thesis_update" => ProposalKind::ThesisUpdate,
        "autonomy_adjustment" => ProposalKind::AutonomyAdjustment,
        "constitution_revision" => ProposalKind::ConstitutionRevision,
        "memory_write" => ProposalKind::MemoryWrite,
        "refactor_module"
        | "modify_schema"
        | "add_route"
        | "add_test"
        | "verify_invariant"
        | "promote_decision"
        | "decompose_goal"
        | "prioritize_work"
        | "record_decision"
        | "register_constraint"
        | "identify_risk"
        | "mark_blocked"
        | "resolve_risk"
        | "restore_progress"
        | "verify_progress"
        | "refresh_working_set"
        | "close_loop"
        | "complete_task"
        | "rollback_change"
        | "detect_affordances"
        | "verify_permissions"
        | "verify_preconditions"
        | "evaluate_dependencies"
        | "estimate_cost"
        | "estimate_latency"
        | "estimate_reliability"
        | "estimate_reversibility"
        | "choose_execution_path"
        | "escalate_authority"
        | "mark_unavailable"
        | "infer_interaction_and_state"
        | "derive_implementation_semantics"
        | "derive_component_tree"
        | "derive_plumbing_requirements"
        | "map_tokens_to_surfaces"
        | "map_states_to_views"
        | "map_bindings_and_validation"
        | "synthesize_completion_checklist" => ProposalKind::OntologyMutation,
        "determine_current_ask"
        | "build_query_scope"
        | "select_relevant_context"
        | "exclude_irrelevant_context"
        | "verify_answer_scope"
        | "record_scope_failure" => ProposalKind::QueryScopeMutation,
        "detect_aliases"
        | "build_resolution_candidates"
        | "resolve_identity"
        | "verify_resolution"
        | "record_supersession" => ProposalKind::ReferenceResolutionMutation,
        "build_projection"
        | "compress_projection"
        | "verify_projection_fidelity"
        | "switch_view_profile" => ProposalKind::ProjectionViewMutation,
        "create_version"
        | "declare_compatibility"
        | "build_migration_plan"
        | "execute_migration"
        | "deprecate_schema_element"
        | "review_governance_change"
        | "verify_post_migration_conformance" => ProposalKind::OntologyGovernanceMutation,
        "establish_identity"
        | "load_role_profile"
        | "verify_capability_profile"
        | "verify_permission_profile"
        | "assign_responsibility"
        | "determine_handoff_boundary"
        | "restore_identity_continuity" => ProposalKind::IdentityModelMutation,
        "derive_structure"
        | "extract_components"
        | "derive_slots"
        | "infer_tokens"
        | "infer_spacing"
        | "map_component_tree"
        | "attach_bindings"
        | "attach_validation"
        | "wire_interaction"
        | "compare_to_reference"
        | "critique_ui" => ProposalKind::VisualModelMutation,
        "ontology_mutation" => ProposalKind::OntologyMutation,
        _ => ProposalKind::FocusChange,
    }
}

fn deterministic_tiebreak_winner(
    proposals: &[focusa_core::types::Proposal],
) -> Option<focusa_core::types::Proposal> {
    proposals.iter().cloned().max_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.created_at.cmp(&b.created_at))
            .then_with(|| a.id.cmp(&b.id))
    })
}

fn parse_autonomy_level(level: &str) -> Option<focusa_core::types::AutonomyLevel> {
    match level {
        "AL0" | "al0" => Some(focusa_core::types::AutonomyLevel::AL0),
        "AL1" | "al1" => Some(focusa_core::types::AutonomyLevel::AL1),
        "AL2" | "al2" => Some(focusa_core::types::AutonomyLevel::AL2),
        "AL3" | "al3" => Some(focusa_core::types::AutonomyLevel::AL3),
        "AL4" | "al4" => Some(focusa_core::types::AutonomyLevel::AL4),
        "AL5" | "al5" => Some(focusa_core::types::AutonomyLevel::AL5),
        _ => None,
    }
}

fn proposal_target_class(kind: ProposalKind) -> &'static str {
    match kind {
        ProposalKind::FocusChange => "focus",
        ProposalKind::ThesisUpdate => "thesis",
        ProposalKind::AutonomyAdjustment => "autonomy",
        ProposalKind::ConstitutionRevision => "constitution",
        ProposalKind::MemoryWrite => "memory",
        ProposalKind::OntologyMutation => "ontology",
        ProposalKind::QueryScopeMutation => "query_scope",
        ProposalKind::ReferenceResolutionMutation => "reference_resolution",
        ProposalKind::ProjectionViewMutation => "projection_view",
        ProposalKind::OntologyGovernanceMutation => "ontology_governance",
        ProposalKind::IdentityModelMutation => "identity_model",
        ProposalKind::VisualModelMutation => "visual_model",
    }
}

fn derived_ontology_applied_kind(kind: ProposalKind, payload: &Value) -> String {
    if let Some(action_type) = payload.get("action_type").and_then(|v| v.as_str()) {
        let parsed = parse_proposal_kind(action_type);
        let parsed_is_ontology_family = matches!(
            parsed,
            ProposalKind::OntologyMutation
                | ProposalKind::QueryScopeMutation
                | ProposalKind::ReferenceResolutionMutation
                | ProposalKind::ProjectionViewMutation
                | ProposalKind::OntologyGovernanceMutation
                | ProposalKind::IdentityModelMutation
                | ProposalKind::VisualModelMutation
        );

        if parsed == kind || (kind == ProposalKind::OntologyMutation && parsed_is_ontology_family) {
            return action_type.to_string();
        }
    }

    match kind {
        ProposalKind::OntologyMutation => "ontology_mutation",
        ProposalKind::QueryScopeMutation => "query_scope_mutation",
        ProposalKind::ReferenceResolutionMutation => "reference_resolution_mutation",
        ProposalKind::ProjectionViewMutation => "projection_view_mutation",
        ProposalKind::OntologyGovernanceMutation => "ontology_governance_mutation",
        ProposalKind::IdentityModelMutation => "identity_model_mutation",
        ProposalKind::VisualModelMutation => "visual_model_mutation",
        ProposalKind::FocusChange => "focus_frame_pushed",
        ProposalKind::ThesisUpdate => "thread_thesis_updated",
        ProposalKind::AutonomyAdjustment => "autonomy_adjusted",
        ProposalKind::ConstitutionRevision => "constitution_loaded",
        ProposalKind::MemoryWrite => "semantic_memory_upserted",
    }
    .to_string()
}

fn submission_audit_event(
    proposal_id: Uuid,
    kind: ProposalKind,
    source: &str,
    payload: &Value,
) -> FocusaEvent {
    if let (Some(link_type), Some(source_id), Some(target_id)) = (
        payload.get("link_type").and_then(|v| v.as_str()),
        payload.get("source_id").and_then(|v| v.as_str()),
        payload.get("target_id").and_then(|v| v.as_str()),
    ) {
        return FocusaEvent::OntologyLinkUpsertProposed {
            proposal_id,
            link_type: link_type.to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            source: source.to_string(),
        };
    }

    match kind {
        ProposalKind::FocusChange => FocusaEvent::OntologyWorkingSetMembershipProposed {
            proposal_id,
            subject: payload
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("focus_change")
                .to_string(),
            operation: "add".to_string(),
            source: source.to_string(),
        },
        ProposalKind::AutonomyAdjustment => FocusaEvent::OntologyStatusChangeProposed {
            proposal_id,
            subject: "autonomy.level".to_string(),
            from_status: None,
            to_status: payload
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            source: source.to_string(),
        },
        ProposalKind::ThesisUpdate => FocusaEvent::OntologyObjectUpsertProposed {
            proposal_id,
            object_type: "thread_thesis".to_string(),
            object_id: payload
                .get("thread_id")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            source: source.to_string(),
        },
        ProposalKind::ConstitutionRevision => FocusaEvent::OntologyObjectUpsertProposed {
            proposal_id,
            object_type: "constitution".to_string(),
            object_id: payload
                .get("version")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            source: source.to_string(),
        },
        ProposalKind::MemoryWrite => FocusaEvent::OntologyObjectUpsertProposed {
            proposal_id,
            object_type: "semantic_memory_entry".to_string(),
            object_id: payload
                .get("key")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            source: source.to_string(),
        },
        ProposalKind::OntologyMutation
        | ProposalKind::QueryScopeMutation
        | ProposalKind::ReferenceResolutionMutation
        | ProposalKind::ProjectionViewMutation
        | ProposalKind::OntologyGovernanceMutation
        | ProposalKind::IdentityModelMutation
        | ProposalKind::VisualModelMutation => FocusaEvent::OntologyObjectUpsertProposed {
            proposal_id,
            object_type: payload
                .get("object_type")
                .and_then(|v| v.as_str())
                .or_else(|| payload.get("target_class").and_then(|v| v.as_str()))
                .unwrap_or(match kind {
                    ProposalKind::QueryScopeMutation => "query_scope",
                    ProposalKind::ReferenceResolutionMutation => "canonical_entity",
                    ProposalKind::ProjectionViewMutation => "projection",
                    ProposalKind::OntologyGovernanceMutation => "governance_decision",
                    ProposalKind::IdentityModelMutation => "agent_identity",
                    ProposalKind::VisualModelMutation => "visual_artifact",
                    _ => "ontology_domain",
                })
                .to_string(),
            object_id: payload
                .get("object_id")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            source: source.to_string(),
        },
    }
}

/// POST /v1/proposals/resolve — run PRE resolution on pending proposals.
async fn resolve_proposals(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let snapshot = state.focusa.read().await.clone();

    let kind_filter = body
        .get("kind")
        .and_then(|v| v.as_str())
        .map(parse_proposal_kind);
    let source_filter = body
        .get("source")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let pending: Vec<_> = snapshot
        .pre
        .proposals
        .iter()
        .filter(|p| matches!(p.status, ProposalStatus::Pending))
        .filter(|p| kind_filter.map(|k| p.kind == k).unwrap_or(true))
        .filter(|p| {
            source_filter
                .as_ref()
                .map(|s| &p.source == s)
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    if pending.is_empty() {
        return Ok(Json(json!({
            "status": "no_proposals",
            "message": "No pending proposals to resolve"
        })));
    }

    let config = focusa_core::pre::resolution::ResolutionConfig::default();
    let window_start = pending
        .iter()
        .map(|p| p.created_at)
        .min()
        .unwrap_or_else(chrono::Utc::now);
    let outcome =
        focusa_core::pre::resolution::resolve_proposals(&pending, &snapshot, &config, window_start);
    let outcome = match outcome {
        focusa_core::pre::resolution::ResolutionOutcome::ClarificationRequired { proposals, reason } => {
            if let Some(winner) = deterministic_tiebreak_winner(&proposals) {
                focusa_core::pre::resolution::ResolutionOutcome::Accepted {
                    score: winner.score,
                    winner,
                    reason: format!("{}; resolved via deterministic tie-break", reason),
                }
            } else {
                focusa_core::pre::resolution::ResolutionOutcome::RejectedAll {
                    reason: "Clarification required but no proposals available".to_string(),
                }
            }
        }
        other => other,
    };

    let mut events_to_emit: Vec<FocusaEvent> = Vec::new();
    let mut visibility_target: Option<(ProposalKind, Value)> = None;

    let result = match outcome {
        focusa_core::pre::resolution::ResolutionOutcome::Accepted {
            winner,
            score,
            reason,
        } => {
            visibility_target = Some((winner.kind, winner.payload.clone()));
            let (applied_kind, mut domain_events): (String, Vec<FocusaEvent>) =
                match winner.kind {
                    ProposalKind::FocusChange => {
                        let reduction =
                            apply_focus_change_proposal(snapshot.clone(), &winner, "api").map_err(
                                |err| (StatusCode::BAD_REQUEST, Json(json!({"error": err}))),
                            )?;
                        ("focus_frame_pushed".to_string(), reduction.emitted_events)
                    }
                    ProposalKind::ThesisUpdate => (
                        "thread_thesis_updated".to_string(),
                        vec![thesis_update_event(&winner).map_err(|err| {
                            (StatusCode::BAD_REQUEST, Json(json!({"error": err})))
                        })?],
                    ),
                    ProposalKind::AutonomyAdjustment => (
                        "autonomy_adjusted".to_string(),
                        vec![autonomy_adjustment_event(&winner).map_err(|err| {
                            (StatusCode::BAD_REQUEST, Json(json!({"error": err})))
                        })?],
                    ),
                    ProposalKind::ConstitutionRevision => (
                        "constitution_loaded".to_string(),
                        vec![constitution_revision_event(&winner).map_err(|err| {
                            (StatusCode::BAD_REQUEST, Json(json!({"error": err})))
                        })?],
                    ),
                    ProposalKind::MemoryWrite => (
                        "semantic_memory_upserted".to_string(),
                        vec![memory_write_event(&winner).map_err(|err| {
                            (StatusCode::BAD_REQUEST, Json(json!({"error": err})))
                        })?],
                    ),
                    ProposalKind::OntologyMutation
                    | ProposalKind::QueryScopeMutation
                    | ProposalKind::ReferenceResolutionMutation
                    | ProposalKind::ProjectionViewMutation
                    | ProposalKind::OntologyGovernanceMutation
                    | ProposalKind::IdentityModelMutation
                    | ProposalKind::VisualModelMutation => {
                        (derived_ontology_applied_kind(winner.kind, &winner.payload), Vec::new())
                    }
                };

            events_to_emit.append(&mut domain_events);
            for proposal in &pending {
                let status = if proposal.id == winner.id {
                    ProposalStatus::Accepted
                } else {
                    ProposalStatus::Rejected
                };
                events_to_emit.push(FocusaEvent::ProposalStatusChanged {
                    proposal_id: proposal.id,
                    status,
                });
                if proposal.id != winner.id {
                    events_to_emit.push(FocusaEvent::OntologyProposalRejected {
                        proposal_id: proposal.id,
                        target_class: proposal_target_class(proposal.kind).to_string(),
                        reason: reason.clone(),
                    });
                }
            }
            events_to_emit.push(FocusaEvent::OntologyVerificationApplied {
                proposal_id: Some(winner.id),
                verification: "pre_resolution".to_string(),
                outcome: "accepted".to_string(),
            });
            events_to_emit.push(FocusaEvent::OntologyProposalPromoted {
                proposal_id: winner.id,
                target_class: proposal_target_class(winner.kind).to_string(),
                applied_kind: applied_kind.clone(),
            });
            if winner.kind == ProposalKind::FocusChange {
                events_to_emit.push(FocusaEvent::OntologyWorkingSetRefreshed {
                    scope: "focus".to_string(),
                    reason: "focus_change accepted".to_string(),
                });
            }
            json!({
                "status": "accepted",
                "winner": winner,
                "score": score,
                "reason": reason,
                "applied_kind": applied_kind,
            })
        }
        focusa_core::pre::resolution::ResolutionOutcome::RejectedAll { reason } => {
            for proposal in &pending {
                events_to_emit.push(FocusaEvent::ProposalStatusChanged {
                    proposal_id: proposal.id,
                    status: ProposalStatus::Rejected,
                });
                events_to_emit.push(FocusaEvent::OntologyProposalRejected {
                    proposal_id: proposal.id,
                    target_class: proposal_target_class(proposal.kind).to_string(),
                    reason: reason.clone(),
                });
            }
            events_to_emit.push(FocusaEvent::OntologyVerificationApplied {
                proposal_id: None,
                verification: "pre_resolution".to_string(),
                outcome: "rejected_all".to_string(),
            });
            json!({
                "status": "rejected_all",
                "reason": reason,
            })
        }
        focusa_core::pre::resolution::ResolutionOutcome::ClarificationRequired {
            proposals,
            reason,
        } => {
            events_to_emit.push(FocusaEvent::OntologyVerificationApplied {
                proposal_id: None,
                verification: "pre_resolution".to_string(),
                outcome: "clarification_required".to_string(),
            });
            json!({
                "status": "clarification_required",
                "proposals": proposals.len(),
                "reason": reason,
            })
        }
    };

    for event in events_to_emit {
        state
            .command_tx
            .send(Action::EmitEvent { event })
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "failed to dispatch proposal resolution event"})),
                )
            })?;
    }

    if let Some((kind, payload)) = visibility_target {
        for _ in 0..120 {
            let visible = {
                let s = state.focusa.read().await;
                match kind {
                    ProposalKind::FocusChange => payload
                        .get("title")
                        .and_then(|v| v.as_str())
                        .map(|title| s.focus_stack.frames.iter().any(|f| f.title == title))
                        .unwrap_or(false),
                    ProposalKind::ThesisUpdate => {
                        let thread_id = payload.get("thread_id").and_then(|v| v.as_str());
                        let primary_intent = payload.get("primary_intent").and_then(|v| v.as_str());
                        match (thread_id, primary_intent) {
                            (Some(thread_id), Some(primary_intent)) => s
                                .threads
                                .iter()
                                .find(|t| t.id.to_string() == thread_id)
                                .map(|t| t.thesis.primary_intent == primary_intent)
                                .unwrap_or(false),
                            _ => false,
                        }
                    }
                    ProposalKind::AutonomyAdjustment => payload
                        .get("level")
                        .and_then(|v| v.as_str())
                        .map(|level| format!("{:?}", s.autonomy.level) == level)
                        .unwrap_or(false),
                    ProposalKind::ConstitutionRevision => payload
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|version| s.constitution.active_version.as_deref() == Some(version))
                        .unwrap_or(false),
                    ProposalKind::MemoryWrite => payload
                        .get("key")
                        .and_then(|v| v.as_str())
                        .zip(payload.get("value").and_then(|v| v.as_str()))
                        .map(|(key, value)| {
                            s.memory
                                .semantic
                                .iter()
                                .any(|m| m.key == key && m.value == value)
                        })
                        .unwrap_or(false),
                    ProposalKind::OntologyMutation
                    | ProposalKind::QueryScopeMutation
                    | ProposalKind::ReferenceResolutionMutation
                    | ProposalKind::ProjectionViewMutation
                    | ProposalKind::OntologyGovernanceMutation
                    | ProposalKind::IdentityModelMutation
                    | ProposalKind::VisualModelMutation => {
                        let object_id = payload.get("object_id").and_then(|v| v.as_str());
                        let source_id = payload.get("source_id").and_then(|v| v.as_str());
                        let target_id = payload.get("target_id").and_then(|v| v.as_str());
                        match (object_id, source_id, target_id) {
                            (Some(object_id), _, _) => s
                                .ontology
                                .objects
                                .iter()
                                .any(|o| o.get("id").and_then(|v| v.as_str()) == Some(object_id)),
                            (None, Some(source_id), Some(target_id)) => s
                                .ontology
                                .links
                                .iter()
                                .any(|l| {
                                    l.get("source_id").and_then(|v| v.as_str())
                                        == Some(source_id)
                                        && l.get("target_id").and_then(|v| v.as_str())
                                            == Some(target_id)
                                }),
                            _ => true,
                        }
                    }
                }
            };
            if visible {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    }

    Ok(Json(result))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/proposals", get(list_proposals).post(submit_proposal))
        .route("/v1/proposals/resolve", post(resolve_proposals))
}
