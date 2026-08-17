//! Workset checkpoint/completion/release transitions — #274 slice 1.
//! The evidence-gated transition DAG: a workset moves from a state to the
//! next ONLY when the gate evidence is present in the ledger. Typed,
//! deterministic, and replayable — no ad-hoc promotion.

use serde::{Deserialize, Serialize};

use crate::workset_ledger::{replay_projection, WorksetDefinition, WorksetEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorksetState {
    Draft,
    Admitted,
    Checkpointed,
    Settled,
    ReleasePending,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionGate {
    pub target: WorksetState,
    pub requires_settled: bool,
    pub requires_all_admitted: bool,
    pub evidence_ref: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionVerdict {
    pub from: WorksetState,
    pub to: WorksetState,
    pub allowed: bool,
    pub reasons: Vec<String>,
}

/// The canonical transition DAG. Each edge names its gate.
pub const TRANSITION_DAG: &[(WorksetState, WorksetState, TransitionGate)] = &[
    (
        WorksetState::Draft,
        WorksetState::Admitted,
        TransitionGate {
            target: WorksetState::Admitted,
            requires_settled: false,
            requires_all_admitted: true,
            evidence_ref: None,
        },
    ),
    (
        WorksetState::Admitted,
        WorksetState::Checkpointed,
        TransitionGate {
            target: WorksetState::Checkpointed,
            requires_settled: false,
            requires_all_admitted: true,
            evidence_ref: Some("checkpoint-receipt"),
        },
    ),
    (
        WorksetState::Checkpointed,
        WorksetState::Settled,
        TransitionGate {
            target: WorksetState::Settled,
            requires_settled: true,
            requires_all_admitted: true,
            evidence_ref: Some("settlement-receipt"),
        },
    ),
    (
        WorksetState::Settled,
        WorksetState::ReleasePending,
        TransitionGate {
            target: WorksetState::ReleasePending,
            requires_settled: true,
            requires_all_admitted: true,
            evidence_ref: Some("release-gate"),
        },
    ),
    (
        WorksetState::ReleasePending,
        WorksetState::Released,
        TransitionGate {
            target: WorksetState::Released,
            requires_settled: true,
            requires_all_admitted: true,
            evidence_ref: Some("release-receipt"),
        },
    ),
];

/// Evaluate a requested transition against the ledger state. A transition
/// is allowed only when the DAG edge exists AND every gate condition holds.
pub fn evaluate_transition(
    definition: &WorksetDefinition,
    events: &[WorksetEvent],
    from: WorksetState,
    to: WorksetState,
) -> Result<TransitionVerdict, String> {
    let projection = replay_projection(definition, events)?;
    let mut verdict = TransitionVerdict {
        from,
        to,
        allowed: false,
        reasons: Vec::new(),
    };
    let edge = TRANSITION_DAG.iter().find(|(f, t, _)| *f == from && *t == to);
    let Some((_, _, gate)) = edge else {
        verdict
            .reasons
            .push(format!("no DAG edge {from:?} → {to:?}"));
        return Ok(verdict);
    };
    if gate.requires_all_admitted
        && projection.requirements.is_empty()
    {
        verdict
            .reasons
            .push("gate requires at least one admitted requirement".to_string());
    }
    if gate.requires_settled && !projection.settled {
        verdict.reasons.push(format!(
            "gate requires settlement; unmet requirements remain"
        ));
    }
    if let Some(required_evidence) = gate.evidence_ref {
        let present = events.iter().any(|event| {
            let evidence = match event {
                WorksetEvent::RequirementAdmitted { evidence_ref, .. }
                | WorksetEvent::RequirementDisposed { evidence_ref, .. }
                | WorksetEvent::MembershipRevised { evidence_ref, .. } => evidence_ref.as_deref(),
                WorksetEvent::CompletionContracted { .. } => None,
            };
            evidence == Some(required_evidence)
        });
        if !present {
            verdict.reasons.push(format!(
                "gate requires evidence {required_evidence}"
            ));
        }
    }
    verdict.allowed = verdict.reasons.is_empty();
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workset_ledger::{
        CompletionContract, RequirementDisposition, WorksetScope,
    };

    fn definition() -> WorksetDefinition {
        WorksetDefinition {
            schema: crate::workset_ledger::WORKSET_LEDGER_SCHEMA.to_string(),
            workset_id: "ws-1".to_string(),
            revision: 1,
            scope: WorksetScope {
                project_root: "/r".to_string(),
                continuity_id: "c".to_string(),
            },
            completion_contract: CompletionContract {
                required_requirement_ids: vec!["r1".to_string()],
                release_gate_ref: Some("release-gate".to_string()),
            },
        }
    }

    fn full_events() -> Vec<WorksetEvent> {
        vec![
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: Some("settlement-receipt".to_string()),
            },
            WorksetEvent::MembershipRevised {
                member_item_id: "m1".to_string(),
                action: crate::workset_ledger::MembershipAction::Admitted,
                evidence_ref: Some("checkpoint-receipt".to_string()),
            },
            WorksetEvent::CompletionContracted {
                contract_digest: "d".to_string(),
            },
            // Release gate evidence: the release-gate ref must be present for
            // the Settled -> ReleasePending edge.
            WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: Some("release-gate".to_string()),
            },
        ]
    }

    #[test]
    fn settled_transition_requires_settlement() {
        let definition = definition();
        let events = vec![WorksetEvent::RequirementAdmitted {
            requirement_id: "r1".to_string(),
            provider_ref: "p1".to_string(),
            evidence_ref: None,
        }];
        let verdict = evaluate_transition(
            &definition,
            &events,
            WorksetState::Checkpointed,
            WorksetState::Settled,
        )
        .unwrap();
        assert!(!verdict.allowed);
        assert!(verdict.reasons.iter().any(|r| r.contains("settlement")));
    }

    #[test]
    fn full_evidence_chain_advances_through_the_dag() {
        let definition = definition();
        let events = full_events();
        let settled = evaluate_transition(
            &definition,
            &events,
            WorksetState::Checkpointed,
            WorksetState::Settled,
        )
        .unwrap();
        assert!(settled.allowed, "reasons: {:?}", settled.reasons);
        let release = evaluate_transition(
            &definition,
            &events,
            WorksetState::Settled,
            WorksetState::ReleasePending,
        )
        .unwrap();
        assert!(release.allowed, "reasons: {:?}", release.reasons);
    }

    #[test]
    fn no_dag_edge_is_rejected() {
        let verdict = evaluate_transition(
            &definition(),
            &[],
            WorksetState::Draft,
            WorksetState::Released,
        )
        .unwrap();
        assert!(!verdict.allowed);
        assert!(verdict.reasons[0].contains("no DAG edge"));
    }

    #[test]
    fn release_pending_requires_the_release_gate_evidence() {
        let definition = definition();
        let mut events = full_events();
        // Remove the release-gate evidence: the CompletionContracted event
        // carries a digest but no evidence ref — add one with the gate ref.
        events.push(WorksetEvent::MembershipRevised {
            member_item_id: "m2".to_string(),
            action: crate::workset_ledger::MembershipAction::Admitted,
            evidence_ref: Some("release-gate".to_string()),
        });
        let verdict = evaluate_transition(
            &definition,
            &events,
            WorksetState::Settled,
            WorksetState::ReleasePending,
        )
        .unwrap();
        assert!(verdict.allowed, "reasons: {:?}", verdict.reasons);
    }
}
