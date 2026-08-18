//! Workset Flow Ledger — #269 slice 1 (Spec 149, operator-authorized
//! 2026-08-16 via #312/#267 directive).
//!
//! Authority separation (#267): the Workset is the approved membership
//! boundary, requirement disposition, and release/completion contract
//! with an immutable history. It is NEVER a second execution graph —
//! CallGraph (#254) owns scheduling and execution. This module owns the
//! append-only ledger with deterministic replay and canonical digests.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const WORKSET_LEDGER_SCHEMA: &str = "focusa.workset_ledger.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorksetDefinition {
    pub schema: String,
    pub workset_id: String,
    pub revision: u64,
    pub scope: WorksetScope,
    pub completion_contract: CompletionContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorksetScope {
    pub project_root: String,
    pub continuity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionContract {
    /// Requirement ids that must be disposed as `met` before the workset
    /// can settle.
    pub required_requirement_ids: Vec<String>,
    pub release_gate_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum WorksetEvent {
    RequirementAdmitted {
        requirement_id: String,
        provider_ref: String,
        evidence_ref: Option<String>,
    },
    RequirementDisposed {
        requirement_id: String,
        disposition: RequirementDisposition,
        evidence_ref: Option<String>,
    },
    MembershipRevised {
        member_item_id: String,
        action: MembershipAction,
        evidence_ref: Option<String>,
    },
    CompletionContracted {
        contract_digest: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementDisposition {
    Met,
    Deferred,
    Waived,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipAction {
    Admitted,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementState {
    pub requirement_id: String,
    pub provider_ref: String,
    pub disposition: Option<RequirementDisposition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorksetProjection {
    pub schema: String,
    pub workset_id: String,
    pub revision: u64,
    pub requirements: BTreeMap<String, RequirementState>,
    pub membership: Vec<String>,
    pub settled: bool,
    pub digest: String,
}

/// Canonical digest over the definition (stable — serde field order).
pub fn workset_digest(definition: &WorksetDefinition) -> String {
    let mut hasher = Sha256::new();
    // Serialize types never fail serialization; the expect documents the
    // invariant instead of silently hashing an empty string.
    let canonical = serde_json::to_string(definition).expect("WorksetDefinition always serializes");
    hasher.update(canonical.as_bytes());
    format!("sha256:{}", hex(&hasher.finalize()))
}

/// Deterministic replay: the same ordered events always produce the same
/// projection. Membership + dispositions only — no execution state.
pub fn replay_projection(
    definition: &WorksetDefinition,
    events: &[WorksetEvent],
) -> Result<WorksetProjection, String> {
    if definition.schema != WORKSET_LEDGER_SCHEMA {
        return Err(format!("unexpected schema {}", definition.schema));
    }
    let mut requirements: BTreeMap<String, RequirementState> = BTreeMap::new();
    let mut membership: Vec<String> = Vec::new();
    for event in events {
        match event {
            WorksetEvent::RequirementAdmitted {
                requirement_id,
                provider_ref,
                ..
            } => {
                requirements.insert(
                    requirement_id.clone(),
                    RequirementState {
                        requirement_id: requirement_id.clone(),
                        provider_ref: provider_ref.clone(),
                        disposition: None,
                    },
                );
            }
            WorksetEvent::RequirementDisposed {
                requirement_id,
                disposition,
                ..
            } => {
                if let Some(state) = requirements.get_mut(requirement_id) {
                    state.disposition = Some(*disposition);
                } else {
                    return Err(format!(
                        "requirement {requirement_id} disposed before admission"
                    ));
                }
            }
            WorksetEvent::MembershipRevised {
                member_item_id,
                action,
                ..
            } => match action {
                MembershipAction::Admitted => {
                    if !membership.contains(member_item_id) {
                        membership.push(member_item_id.clone());
                    }
                }
                MembershipAction::Removed => {
                    membership.retain(|id| id != member_item_id);
                }
            },
            WorksetEvent::CompletionContracted { .. } => {}
        }
    }
    let settled = definition
        .completion_contract
        .required_requirement_ids
        .iter()
        .all(|required| {
            requirements
                .get(required)
                .and_then(|state| state.disposition)
                == Some(RequirementDisposition::Met)
        });
    let digest = workset_digest(definition);
    Ok(WorksetProjection {
        schema: "focusa.workset_projection.v1".to_string(),
        workset_id: definition.workset_id.clone(),
        revision: definition.revision,
        requirements,
        membership,
        settled,
        digest,
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Bridge (docs/170 gap B): met requirements supply their evidence
/// refs as completion-claim coverage — the workset disposition and the
/// completion verdict share one evidence vocabulary.
pub fn met_requirement_evidence(
    definition: &WorksetDefinition,
    events: &[WorksetEvent],
) -> Result<Vec<String>, String> {
    let projection = replay_projection(definition, events)?;
    let mut evidence = Vec::new();
    for event in events {
        if let WorksetEvent::RequirementDisposed {
            requirement_id,
            disposition,
            evidence_ref,
        } = event
        {
            if *disposition == RequirementDisposition::Met
                && projection
                    .requirements
                    .get(requirement_id)
                    .map(|state| state.disposition == Some(RequirementDisposition::Met))
                    .unwrap_or(false)
            {
                if let Some(reference) = evidence_ref {
                    evidence.push(reference.clone());
                }
            }
        }
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition() -> WorksetDefinition {
        WorksetDefinition {
            schema: WORKSET_LEDGER_SCHEMA.to_string(),
            workset_id: "ws-1".to_string(),
            revision: 1,
            scope: WorksetScope {
                project_root: "/root/proj".to_string(),
                continuity_id: "cont-1".to_string(),
            },
            completion_contract: CompletionContract {
                required_requirement_ids: vec!["r1".to_string(), "r2".to_string()],
                release_gate_ref: Some("release-gate-1".to_string()),
            },
        }
    }

    #[test]
    fn replay_is_deterministic_and_settles_on_met_requirements() {
        let events = vec![
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: Some("e1".to_string()),
            },
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r2".to_string(),
                provider_ref: "p2".to_string(),
                evidence_ref: Some("e2".to_string()),
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: Some("e3".to_string()),
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r2".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: Some("e4".to_string()),
            },
            WorksetEvent::MembershipRevised {
                member_item_id: "m1".to_string(),
                action: MembershipAction::Admitted,
                evidence_ref: None,
            },
        ];
        let definition = definition();
        let first = replay_projection(&definition, &events).unwrap();
        let second = replay_projection(&definition, &events).unwrap();
        assert_eq!(first, second, "replay must be deterministic");
        assert!(first.settled, "all required requirements met");
        assert_eq!(first.membership, vec!["m1"]);
        assert!(first.digest.starts_with("sha256:"));
    }

    #[test]
    fn disposition_before_admission_is_rejected() {
        let events = vec![WorksetEvent::RequirementDisposed {
            requirement_id: "r1".to_string(),
            disposition: RequirementDisposition::Met,
            evidence_ref: None,
        }];
        assert!(replay_projection(&definition(), &events).is_err());
    }

    #[test]
    fn deferred_requirement_blocks_settlement() {
        let events = vec![
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            },
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r2".to_string(),
                provider_ref: "p2".to_string(),
                evidence_ref: None,
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: None,
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r2".to_string(),
                disposition: RequirementDisposition::Deferred,
                evidence_ref: None,
            },
        ];
        let projection = replay_projection(&definition(), &events).unwrap();
        assert!(!projection.settled);
    }

    #[test]
    fn membership_removal_is_reflected() {
        let events = vec![
            WorksetEvent::MembershipRevised {
                member_item_id: "m1".to_string(),
                action: MembershipAction::Admitted,
                evidence_ref: None,
            },
            WorksetEvent::MembershipRevised {
                member_item_id: "m1".to_string(),
                action: MembershipAction::Removed,
                evidence_ref: None,
            },
        ];
        let projection = replay_projection(&definition(), &events).unwrap();
        assert!(projection.membership.is_empty());
    }
}

#[cfg(test)]
mod bridge_tests {
    use super::*;
    use crate::workset_ledger::{CompletionContract, WorksetScope};

    #[test]
    fn met_requirements_supply_their_evidence() {
        let definition = WorksetDefinition {
            schema: WORKSET_LEDGER_SCHEMA.to_string(),
            workset_id: "ws-1".to_string(),
            revision: 1,
            scope: WorksetScope {
                project_root: "/r".to_string(),
                continuity_id: "c".to_string(),
            },
            completion_contract: CompletionContract {
                required_requirement_ids: vec!["r1".to_string()],
                release_gate_ref: None,
            },
        };
        let events = vec![
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: Some("docs/evidence/r1.md".to_string()),
            },
        ];
        let evidence = met_requirement_evidence(&definition, &events).unwrap();
        assert_eq!(evidence, vec!["docs/evidence/r1.md"]);
    }
}
