//! Workset provider admission + reconciliation — #270 slice 1.
//! Providers project requirement state; the workset reconciles their
//! projections against the canonical ledger — provider claims never
//! override the append-only history (authority separation: #267).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::workset_ledger::{
    replay_projection, RequirementDisposition, WorksetDefinition, WorksetEvent,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderProjection {
    pub provider_ref: String,
    pub requirement_dispositions: BTreeMap<String, RequirementDisposition>,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub provider_ref: String,
    pub agreements: Vec<String>,
    pub conflicts: Vec<String>,
    pub reconciled: bool,
}

/// Reconcile a provider projection against the canonical replay. A
/// provider may only CLAIM dispositions; the ledger wins every conflict.
pub fn reconcile_provider(
    definition: &WorksetDefinition,
    events: &[WorksetEvent],
    provider: &ProviderProjection,
) -> Result<ReconciliationReport, String> {
    let canonical = replay_projection(definition, events)?;
    let mut agreements = Vec::new();
    let mut conflicts = Vec::new();
    for (requirement_id, claimed) in &provider.requirement_dispositions {
        match canonical.requirements.get(requirement_id) {
            Some(state) => match state.disposition {
                Some(actual) if actual == *claimed => {
                    agreements.push(requirement_id.clone());
                }
                Some(actual) => {
                    conflicts.push(format!(
                        "{requirement_id}: provider claims {claimed:?}, ledger has {actual:?}"
                    ));
                }
                None => {
                    conflicts.push(format!(
                        "{requirement_id}: provider claims {claimed:?}, ledger has no disposition"
                    ));
                }
            },
            None => {
                conflicts.push(format!(
                    "{requirement_id}: provider claims {claimed:?}, requirement not admitted"
                ));
            }
        }
    }
    Ok(ReconciliationReport {
        provider_ref: provider.provider_ref.clone(),
        agreements,
        conflicts,
        reconciled: conflicts.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workset_ledger::{
        CompletionContract, WorksetScope,
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
                release_gate_ref: None,
            },
        }
    }

    #[test]
    fn agreeing_provider_reconciles() {
        let events = vec![
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: RequirementDisposition::Met,
                evidence_ref: None,
            },
        ];
        let mut dispositions = BTreeMap::new();
        dispositions.insert("r1".to_string(), RequirementDisposition::Met);
        let provider = ProviderProjection {
            provider_ref: "p1".to_string(),
            requirement_dispositions: dispositions,
            generated_at: "t".to_string(),
        };
        let report = reconcile_provider(&definition(), &events, &provider).unwrap();
        assert!(report.reconciled);
        assert_eq!(report.agreements, vec!["r1"]);
    }

    #[test]
    fn conflicting_provider_does_not_override_the_ledger() {
        let events = vec![
            WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            },
            WorksetEvent::RequirementDisposed {
                requirement_id: "r1".to_string(),
                disposition: RequirementDisposition::Deferred,
                evidence_ref: None,
            },
        ];
        let mut dispositions = BTreeMap::new();
        dispositions.insert("r1".to_string(), RequirementDisposition::Met);
        let provider = ProviderProjection {
            provider_ref: "p1".to_string(),
            requirement_dispositions: dispositions,
            generated_at: "t".to_string(),
        };
        let report = reconcile_provider(&definition(), &events, &provider).unwrap();
        assert!(!report.reconciled);
        assert_eq!(report.conflicts.len(), 1);
    }

    #[test]
    fn claims_for_unadmitted_requirements_conflict() {
        let mut dispositions = BTreeMap::new();
        dispositions.insert("ghost".to_string(), RequirementDisposition::Met);
        let provider = ProviderProjection {
            provider_ref: "p1".to_string(),
            requirement_dispositions: dispositions,
            generated_at: "t".to_string(),
        };
        let report = reconcile_provider(&definition(), &[], &provider).unwrap();
        assert!(!report.reconciled);
        assert!(report.conflicts[0].contains("not admitted"));
    }
}
