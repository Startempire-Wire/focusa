//! Workset bounded context — #272 slice 1. Consumers preload a BOUNDED
//! projection (never the raw ledger): stamp + requirement dispositions +
//! membership + optional Workpoint/CallGraph bindings. The binding
//! carries reference ids only — execution stays in CallGraph/Workpoint.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::workset_freshness::{FreshnessStamp, canonical_stamp};
use crate::workset_ledger::{WorksetDefinition, WorksetEvent};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorksetBindings {
    pub workpoint_ref: Option<String>,
    pub callgraph_run_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorksetContextPacket {
    pub schema: String,
    pub stamp: FreshnessStamp,
    pub requirement_dispositions: BTreeMap<String, Option<String>>,
    pub membership: Vec<String>,
    pub settled: bool,
    pub bindings: WorksetBindings,
}

/// Build the bounded preload packet — explicit, digest-anchored, and
/// never the raw event list (bounded-context discipline).
pub fn build_context_packet(
    definition: &WorksetDefinition,
    events: &[WorksetEvent],
    bindings: WorksetBindings,
    max_requirements: usize,
) -> Result<WorksetContextPacket, String> {
    let stamp = canonical_stamp(definition, events)?;
    let projection = crate::workset_ledger::replay_projection(definition, events)?;
    let mut dispositions = BTreeMap::new();
    for (id, state) in projection.requirements.iter().take(max_requirements) {
        dispositions.insert(
            id.clone(),
            state.disposition.map(|d| format!("{d:?}").to_lowercase()),
        );
    }
    Ok(WorksetContextPacket {
        schema: "focusa.workset_context_packet.v1".to_string(),
        stamp,
        requirement_dispositions: dispositions,
        membership: projection.membership,
        settled: projection.settled,
        bindings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workset_ledger::{CompletionContract, RequirementDisposition, WorksetScope};

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
    fn packet_is_bounded_and_stamped() {
        let definition = definition();
        let events = vec![WorksetEvent::RequirementAdmitted {
            requirement_id: "r1".to_string(),
            provider_ref: "p1".to_string(),
            evidence_ref: None,
        }];
        let packet = build_context_packet(
            &definition,
            &events,
            WorksetBindings {
                workpoint_ref: Some("wp-1".to_string()),
                callgraph_run_ref: Some("run-1".to_string()),
            },
            10,
        )
        .unwrap();
        assert_eq!(packet.stamp.event_count, 1);
        assert_eq!(packet.requirement_dispositions.len(), 1);
        assert_eq!(packet.bindings.workpoint_ref.as_deref(), Some("wp-1"));
        assert!(!packet.settled);
    }

    #[test]
    fn max_requirements_bounds_the_packet() {
        let definition = definition();
        let mut events = Vec::new();
        for i in 0..20 {
            events.push(WorksetEvent::RequirementAdmitted {
                requirement_id: format!("r{i}"),
                provider_ref: "p".to_string(),
                evidence_ref: None,
            });
        }
        let packet = build_context_packet(
            &definition,
            &events,
            WorksetBindings {
                workpoint_ref: None,
                callgraph_run_ref: None,
            },
            5,
        )
        .unwrap();
        assert_eq!(packet.requirement_dispositions.len(), 5);
        // The stamp still reflects the FULL ledger (integrity), while the
        // packet stays bounded (context).
        assert_eq!(packet.stamp.event_count, 20);
    }

    #[test]
    fn dispositions_survive_replay() {
        let definition = definition();
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
        let packet = build_context_packet(
            &definition,
            &events,
            WorksetBindings {
                workpoint_ref: None,
                callgraph_run_ref: None,
            },
            10,
        )
        .unwrap();
        assert!(packet.settled);
        assert_eq!(
            packet
                .requirement_dispositions
                .get("r1")
                .and_then(|d| d.as_deref()),
            Some("met")
        );
    }
}
