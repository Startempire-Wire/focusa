//! Working-set freshness + cross-consumer integrity — #268 slice 1
//! (Spec 49). A consumer's working set is fresh exactly when its stamp
//! matches the canonical ledger stamp; any drift flags the consumer as
//! stale with the exact divergence. No consumer mutates the ledger to
//! "refresh" — they re-read.

use serde::{Deserialize, Serialize};

use crate::workset_ledger::{
    replay_projection, workset_digest, WorksetDefinition, WorksetEvent,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessStamp {
    pub workset_id: String,
    pub revision: u64,
    pub event_count: usize,
    pub digest: String,
    pub settled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessVerdict {
    pub fresh: bool,
    pub consumer_stamp: FreshnessStamp,
    pub canonical_stamp: FreshnessStamp,
    pub divergences: Vec<String>,
}

/// The canonical stamp for the current ledger state.
pub fn canonical_stamp(
    definition: &WorksetDefinition,
    events: &[WorksetEvent],
) -> Result<FreshnessStamp, String> {
    let projection = replay_projection(definition, events)?;
    Ok(FreshnessStamp {
        workset_id: definition.workset_id.clone(),
        revision: definition.revision,
        event_count: events.len(),
        digest: projection.digest,
        settled: projection.settled,
    })
}

/// Compare a consumer's held stamp against the canonical stamp. Fresh
/// only when revision, event count, and digest all agree.
pub fn check_freshness(
    consumer: &FreshnessStamp,
    canonical: &FreshnessStamp,
) -> FreshnessVerdict {
    let mut divergences = Vec::new();
    if consumer.revision != canonical.revision {
        divergences.push(format!(
            "revision: consumer {} vs canonical {}",
            consumer.revision, canonical.revision
        ));
    }
    if consumer.event_count != canonical.event_count {
        divergences.push(format!(
            "event_count: consumer {} vs canonical {}",
            consumer.event_count, canonical.event_count
        ));
    }
    if consumer.digest != canonical.digest {
        divergences.push("digest diverged".to_string());
    }
    FreshnessVerdict {
        fresh: divergences.is_empty(),
        consumer_stamp: consumer.clone(),
        canonical_stamp: canonical.clone(),
        divergences,
    }
}

/// Cross-consumer integrity: two consumers agree exactly when their
/// stamps are identical — no per-consumer interpretation of freshness.
pub fn consumers_agree(a: &FreshnessStamp, b: &FreshnessStamp) -> bool {
    a == b
}

pub fn derive_definition_digest(definition: &WorksetDefinition) -> String {
    workset_digest(definition)
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
                release_gate_ref: None,
            },
        }
    }

    #[test]
    fn fresh_consumer_stamp_matches_canonical() {
        let definition = definition();
        let events = vec![WorksetEvent::RequirementAdmitted {
            requirement_id: "r1".to_string(),
            provider_ref: "p1".to_string(),
            evidence_ref: None,
        }];
        let canonical = canonical_stamp(&definition, &events).unwrap();
        let consumer = canonical.clone();
        let verdict = check_freshness(&consumer, &canonical);
        assert!(verdict.fresh);
        assert!(verdict.divergences.is_empty());
    }

    #[test]
    fn stale_consumer_is_flagged_with_divergences() {
        let definition = definition();
        let canonical = canonical_stamp(
            &definition,
            &[WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            }],
        )
        .unwrap();
        let consumer = FreshnessStamp {
            workset_id: "ws-1".to_string(),
            revision: 1,
            event_count: 0,
            digest: "sha256:stale".to_string(),
            settled: false,
        };
        let verdict = check_freshness(&consumer, &canonical);
        assert!(!verdict.fresh);
        assert!(verdict.divergences.iter().any(|d| d.contains("event_count")));
        assert!(verdict.divergences.iter().any(|d| d == "digest diverged"));
    }

    #[test]
    fn two_consumers_agree_only_on_identical_stamps() {
        let definition = definition();
        let canonical = canonical_stamp(
            &definition,
            &[WorksetEvent::RequirementAdmitted {
                requirement_id: "r1".to_string(),
                provider_ref: "p1".to_string(),
                evidence_ref: None,
            }],
        )
        .unwrap();
        assert!(consumers_agree(&canonical, &canonical));
        let mut drifted = canonical.clone();
        drifted.digest = "sha256:drift".to_string();
        assert!(!consumers_agree(&canonical, &drifted));
    }

    #[test]
    fn settled_flag_rides_the_stamp() {
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
        let stamp = canonical_stamp(&definition, &events).unwrap();
        assert!(stamp.settled);
    }

    #[test]
    fn definition_digest_is_stable() {
        let definition = definition();
        assert_eq!(
            derive_definition_digest(&definition),
            derive_definition_digest(&definition)
        );
    }
}
