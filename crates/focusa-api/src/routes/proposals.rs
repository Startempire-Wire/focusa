//! PRE (Proposal Resolution Engine) routes.

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::{get, post}};
use focusa_core::reducer;
use focusa_core::types::{Action, EventLogEntry, FocusaEvent, ProposalKind, ProposalStatus, SignalOrigin};
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

    let kind = match kind_str {
        "focus_change" => ProposalKind::FocusChange,
        "thesis_update" => ProposalKind::ThesisUpdate,
        "autonomy_adjustment" => ProposalKind::AutonomyAdjustment,
        "constitution_revision" => ProposalKind::ConstitutionRevision,
        "memory_write" => ProposalKind::MemoryWrite,
        _ => ProposalKind::FocusChange,
    };

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

    let mut proposal_id = None;
    for _ in 0..40 {
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

    if let Some(proposal_id) = proposal_id
        && let Ok(machine_id) = state.persistence.machine_id()
    {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: chrono::Utc::now(),
            event: submission_audit_event(proposal_id, kind, source, &payload_for_audit),
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
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let tags = winner
        .payload
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
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

fn apply_thesis_update_proposal(
    state: &mut focusa_core::types::FocusaState,
    winner: &focusa_core::types::Proposal,
) -> Result<(), String> {
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
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let open_questions = winner
        .payload
        .get("open_questions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let assumptions = winner
        .payload
        .get("assumptions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let sources = winner
        .payload
        .get("sources")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
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

    let requested_thread_id = winner
        .payload
        .get("thread_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    let thread_index = if let Some(thread_id) = requested_thread_id {
        state
            .threads
            .iter()
            .position(|t| t.id == thread_id)
            .ok_or_else(|| format!("thread {} not found for thesis update", thread_id))?
    } else {
        state
            .threads
            .iter()
            .position(|t| t.status == focusa_core::types::ThreadStatus::Active)
            .or_else(|| (!state.threads.is_empty()).then_some(0))
            .ok_or_else(|| "no thread available for thesis update".to_string())?
    };
    let thread = &mut state.threads[thread_index];

    thread.thesis.primary_intent = primary_intent;
    thread.thesis.secondary_goals = secondary_goals;
    thread.thesis.open_questions = open_questions;
    thread.thesis.assumptions = assumptions;
    thread.thesis.sources = sources;
    thread.thesis.confidence.score = confidence_score;
    thread.thesis.confidence.rationale = confidence_rationale;
    thread.thesis.updated_at = Some(chrono::Utc::now());
    thread.updated_at = chrono::Utc::now();
    Ok(())
}

fn apply_memory_write_proposal(
    state: &mut focusa_core::types::FocusaState,
    winner: &focusa_core::types::Proposal,
) -> Result<FocusaEvent, String> {
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

    let event = focusa_core::memory::semantic::upsert(
        &mut state.memory,
        key,
        value,
        focusa_core::types::MemorySource::User,
    );

    Ok(match event {
        FocusaEvent::SemanticMemoryUpserted { key, value, .. } => FocusaEvent::SemanticMemoryUpserted {
            key,
            value,
            source: source.to_string(),
        },
        other => other,
    })
}

fn apply_autonomy_adjustment_proposal(
    state: &mut focusa_core::types::FocusaState,
    winner: &focusa_core::types::Proposal,
) -> Result<(), String> {
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
    let ttl_seconds = winner
        .payload
        .get("ttl_seconds")
        .and_then(|v| v.as_i64());
    let ttl = ttl_seconds.map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs));
    let reason = winner
        .payload
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("proposal resolution accepted");

    focusa_core::autonomy::grant_level(&mut state.autonomy, level, scope, ttl, reason);
    Ok(())
}

fn apply_constitution_revision_proposal(
    state: &mut focusa_core::types::FocusaState,
    winner: &focusa_core::types::Proposal,
) -> Result<(), String> {
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
                    v.as_str().map(|text| focusa_core::types::ConstitutionPrinciple {
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
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let expression_rules: Vec<String> = winner
        .payload
        .get("expression_rules")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    if principles.is_empty() && safety_rules.is_empty() && expression_rules.is_empty() {
        return Err("constitution_revision requires at least one principle/safety/expression rule".to_string());
    }

    focusa_core::constitution::create_version(
        &mut state.constitution,
        agent_id,
        &version,
        principles,
        safety_rules,
        expression_rules,
    );
    focusa_core::constitution::activate_version(&mut state.constitution, &version)?;
    Ok(())
}

fn parse_proposal_kind(kind_str: &str) -> ProposalKind {
    match kind_str {
        "focus_change" => ProposalKind::FocusChange,
        "thesis_update" => ProposalKind::ThesisUpdate,
        "autonomy_adjustment" => ProposalKind::AutonomyAdjustment,
        "constitution_revision" => ProposalKind::ConstitutionRevision,
        "memory_write" => ProposalKind::MemoryWrite,
        _ => ProposalKind::FocusChange,
    }
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
    }
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
            object_id: payload.get("thread_id").and_then(|v| v.as_str()).map(str::to_string),
            source: source.to_string(),
        },
        ProposalKind::ConstitutionRevision => FocusaEvent::OntologyObjectUpsertProposed {
            proposal_id,
            object_type: "constitution".to_string(),
            object_id: payload.get("version").and_then(|v| v.as_str()).map(str::to_string),
            source: source.to_string(),
        },
        ProposalKind::MemoryWrite => FocusaEvent::OntologyObjectUpsertProposed {
            proposal_id,
            object_type: "semantic_memory_entry".to_string(),
            object_id: payload.get("key").and_then(|v| v.as_str()).map(str::to_string),
            source: source.to_string(),
        },
    }
}

/// POST /v1/proposals/resolve — run PRE resolution on pending proposals.
async fn resolve_proposals(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let machine_id = state.persistence.machine_id().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("failed to get machine id: {}", e)})),
        )
    })?;

    let snapshot = state.focusa.read().await.clone();

    let kind_filter = body
        .get("kind")
        .and_then(|v| v.as_str())
        .map(parse_proposal_kind);

    let pending: Vec<_> = snapshot
        .pre
        .proposals
        .iter()
        .filter(|p| matches!(p.status, ProposalStatus::Pending))
        .filter(|p| kind_filter.map(|k| p.kind == k).unwrap_or(true))
        .cloned()
        .collect();

    if pending.is_empty() {
        return Ok(Json(json!({
            "status": "no_proposals",
            "message": "No pending proposals to resolve"
        })));
    }

    let config = focusa_core::pre::resolution::ResolutionConfig::default();
    let window_start = chrono::Utc::now();
    let outcome = focusa_core::pre::resolution::resolve_proposals(&pending, &snapshot, &config, window_start);

    let mut next_state = snapshot.clone();
    let mut applied_events: Vec<FocusaEvent> = Vec::new();

    let result = match outcome {
        focusa_core::pre::resolution::ResolutionOutcome::Accepted { winner, score, reason } => {
            match winner.kind {
                ProposalKind::FocusChange => {
                    let reduction = apply_focus_change_proposal(next_state.clone(), &winner, &machine_id);
                    match reduction {
                        Ok(reduction) => {
                            next_state = reduction.new_state;
                            applied_events.extend(reduction.emitted_events.clone());
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id {
                                    proposal.status = ProposalStatus::Accepted;
                                } else if matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            applied_events.push(FocusaEvent::OntologyVerificationApplied {
                                proposal_id: Some(winner.id),
                                verification: "pre_resolution".to_string(),
                                outcome: "accepted".to_string(),
                            });
                            applied_events.push(FocusaEvent::OntologyProposalPromoted {
                                proposal_id: winner.id,
                                target_class: proposal_target_class(winner.kind).to_string(),
                                applied_kind: "focus_frame_pushed".to_string(),
                            });
                            applied_events.push(FocusaEvent::OntologyWorkingSetRefreshed {
                                scope: "focus".to_string(),
                                reason: "focus_change accepted".to_string(),
                            });
                            json!({
                                "status": "accepted",
                                "winner": winner,
                                "score": score,
                                "reason": reason,
                                "applied_kind": "focus_frame_pushed"
                            })
                        }
                        Err(err) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id || matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            json!({
                                "status": "rejected_all",
                                "reason": format!("accepted proposal could not be canonically applied: {}", err),
                            })
                        }
                    }
                }
                ProposalKind::ThesisUpdate => {
                    match apply_thesis_update_proposal(&mut next_state, &winner) {
                        Ok(()) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id {
                                    proposal.status = ProposalStatus::Accepted;
                                } else if matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            applied_events.push(FocusaEvent::OntologyVerificationApplied {
                                proposal_id: Some(winner.id),
                                verification: "pre_resolution".to_string(),
                                outcome: "accepted".to_string(),
                            });
                            applied_events.push(FocusaEvent::OntologyProposalPromoted {
                                proposal_id: winner.id,
                                target_class: proposal_target_class(winner.kind).to_string(),
                                applied_kind: "thread_thesis_updated".to_string(),
                            });
                            json!({
                                "status": "accepted",
                                "winner": winner,
                                "score": score,
                                "reason": reason,
                                "applied_kind": "thread_thesis_updated"
                            })
                        }
                        Err(err) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id || matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            json!({
                                "status": "rejected_all",
                                "reason": format!("accepted proposal could not be canonically applied: {}", err),
                            })
                        }
                    }
                }
                ProposalKind::MemoryWrite => {
                    match apply_memory_write_proposal(&mut next_state, &winner) {
                        Ok(event) => {
                            applied_events.push(event);
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id {
                                    proposal.status = ProposalStatus::Accepted;
                                } else if matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            applied_events.push(FocusaEvent::OntologyVerificationApplied {
                                proposal_id: Some(winner.id),
                                verification: "pre_resolution".to_string(),
                                outcome: "accepted".to_string(),
                            });
                            applied_events.push(FocusaEvent::OntologyProposalPromoted {
                                proposal_id: winner.id,
                                target_class: proposal_target_class(winner.kind).to_string(),
                                applied_kind: "semantic_memory_upserted".to_string(),
                            });
                            json!({
                                "status": "accepted",
                                "winner": winner,
                                "score": score,
                                "reason": reason,
                                "applied_kind": "semantic_memory_upserted"
                            })
                        }
                        Err(err) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id || matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            json!({
                                "status": "rejected_all",
                                "reason": format!("accepted proposal could not be canonically applied: {}", err),
                            })
                        }
                    }
                }
                ProposalKind::AutonomyAdjustment => {
                    match apply_autonomy_adjustment_proposal(&mut next_state, &winner) {
                        Ok(()) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id {
                                    proposal.status = ProposalStatus::Accepted;
                                } else if matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            applied_events.push(FocusaEvent::OntologyVerificationApplied {
                                proposal_id: Some(winner.id),
                                verification: "pre_resolution".to_string(),
                                outcome: "accepted".to_string(),
                            });
                            applied_events.push(FocusaEvent::OntologyProposalPromoted {
                                proposal_id: winner.id,
                                target_class: proposal_target_class(winner.kind).to_string(),
                                applied_kind: "autonomy_level_granted".to_string(),
                            });
                            json!({
                                "status": "accepted",
                                "winner": winner,
                                "score": score,
                                "reason": reason,
                                "applied_kind": "autonomy_level_granted"
                            })
                        }
                        Err(err) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id || matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            json!({
                                "status": "rejected_all",
                                "reason": format!("accepted proposal could not be canonically applied: {}", err),
                            })
                        }
                    }
                }
                ProposalKind::ConstitutionRevision => {
                    match apply_constitution_revision_proposal(&mut next_state, &winner) {
                        Ok(()) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id {
                                    proposal.status = ProposalStatus::Accepted;
                                } else if matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            applied_events.push(FocusaEvent::OntologyVerificationApplied {
                                proposal_id: Some(winner.id),
                                verification: "pre_resolution".to_string(),
                                outcome: "accepted".to_string(),
                            });
                            applied_events.push(FocusaEvent::OntologyProposalPromoted {
                                proposal_id: winner.id,
                                target_class: proposal_target_class(winner.kind).to_string(),
                                applied_kind: "constitution_version_activated".to_string(),
                            });
                            json!({
                                "status": "accepted",
                                "winner": winner,
                                "score": score,
                                "reason": reason,
                                "applied_kind": "constitution_version_activated"
                            })
                        }
                        Err(err) => {
                            for proposal in &mut next_state.pre.proposals {
                                if proposal.id == winner.id || matches!(proposal.status, ProposalStatus::Pending) {
                                    proposal.status = ProposalStatus::Rejected;
                                }
                            }
                            json!({
                                "status": "rejected_all",
                                "reason": format!("accepted proposal could not be canonically applied: {}", err),
                            })
                        }
                    }
                }
            }
        }
        focusa_core::pre::resolution::ResolutionOutcome::RejectedAll { reason } => {
            for proposal in &mut next_state.pre.proposals {
                if matches!(proposal.status, ProposalStatus::Pending)
                    && kind_filter.map(|k| proposal.kind == k).unwrap_or(true)
                {
                    applied_events.push(FocusaEvent::OntologyProposalRejected {
                        proposal_id: proposal.id,
                        target_class: proposal_target_class(proposal.kind).to_string(),
                        reason: reason.clone(),
                    });
                    proposal.status = ProposalStatus::Rejected;
                }
            }
            applied_events.push(FocusaEvent::OntologyVerificationApplied {
                proposal_id: None,
                verification: "pre_resolution".to_string(),
                outcome: "rejected_all".to_string(),
            });
            json!({
                "status": "rejected_all",
                "reason": reason,
            })
        }
        focusa_core::pre::resolution::ResolutionOutcome::ClarificationRequired { proposals, reason } => {
            applied_events.push(FocusaEvent::OntologyVerificationApplied {
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

    for event in applied_events {
        let entry = EventLogEntry {
            id: Uuid::now_v7(),
            timestamp: chrono::Utc::now(),
            event,
            correlation_id: Some("api:resolve_proposals".to_string()),
            origin: SignalOrigin::Cli,
            machine_id: Some(machine_id.clone()),
            instance_id: None,
            session_id: next_state.session.as_ref().map(|s| s.session_id),
            thread_id: None,
            is_observation: false,
        };
        if let Err(e) = state.persistence.append_event(&entry) {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("failed to persist proposal event: {}", e)})),
            ));
        }
    }

    state.persistence.save_state(&next_state).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("failed to save proposal resolution state: {}", e)})),
        )
    })?;

    {
        let mut shared = state.focusa.write().await;
        *shared = next_state;
    }

    Ok(Json(result))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/proposals", get(list_proposals).post(submit_proposal))
        .route("/v1/proposals/resolve", post(resolve_proposals))
}
