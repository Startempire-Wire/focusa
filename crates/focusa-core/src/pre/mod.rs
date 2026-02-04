//! Proposal Resolution Engine (PRE) — docs/41-proposal-resolution-engine.md
//!
//! Timestamped async requests for state change.
//! Proposals are scored, prioritized, and resolved.
//! Resolution window: configurable (default 5000ms).
//! Only accepted proposals trigger reducer events.

use crate::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Submit a new proposal.
pub fn submit(
    state: &mut PreState,
    kind: ProposalKind,
    source: &str,
    payload: serde_json::Value,
    deadline_ms: u64,
) -> Uuid {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let deadline = now + chrono::Duration::milliseconds(deadline_ms as i64);

    state.proposals.push(Proposal {
        id,
        kind,
        source: source.into(),
        created_at: now,
        deadline,
        payload,
        score: 0.0,
        status: ProposalStatus::Pending,
    });

    id
}

/// Score a proposal (0.0 to 1.0).
pub fn score_proposal(state: &mut PreState, proposal_id: Uuid, score: f64) -> Result<(), String> {
    let p = state.proposals.iter_mut().find(|p| p.id == proposal_id)
        .ok_or_else(|| format!("Proposal {} not found", proposal_id))?;
    if p.status != ProposalStatus::Pending {
        return Err(format!("Proposal {} is {:?}, not Pending", proposal_id, p.status));
    }
    p.score = score.clamp(0.0, 1.0);
    Ok(())
}

/// Resolve all pending proposals within the resolution window.
/// Returns accepted proposal IDs.
pub fn resolve(state: &mut PreState, acceptance_threshold: f64) -> Vec<Uuid> {
    let now = Utc::now();
    let mut accepted = Vec::new();

    for p in &mut state.proposals {
        if p.status != ProposalStatus::Pending {
            continue;
        }
        if now > p.deadline {
            p.status = ProposalStatus::Expired;
            continue;
        }
        if p.score >= acceptance_threshold {
            p.status = ProposalStatus::Accepted;
            accepted.push(p.id);
        }
    }

    accepted
}

/// Reject a specific proposal.
pub fn reject(state: &mut PreState, proposal_id: Uuid) -> Result<(), String> {
    let p = state.proposals.iter_mut().find(|p| p.id == proposal_id)
        .ok_or_else(|| format!("Proposal {} not found", proposal_id))?;
    p.status = ProposalStatus::Rejected;
    Ok(())
}

/// Garbage-collect expired/resolved proposals older than retention period.
pub fn gc(state: &mut PreState, retention_secs: i64) {
    let cutoff = Utc::now() - chrono::Duration::seconds(retention_secs);
    state.proposals.retain(|p| {
        p.status == ProposalStatus::Pending || p.created_at > cutoff
    });
}

/// Count pending proposals.
pub fn pending_count(state: &PreState) -> usize {
    state.proposals.iter().filter(|p| p.status == ProposalStatus::Pending).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_and_resolve() {
        let mut state = PreState { proposals: vec![], resolution_window_ms: 5000 };
        let id = submit(&mut state, ProposalKind::FocusChange, "test", serde_json::json!({}), 60000);
        score_proposal(&mut state, id, 0.8).unwrap();
        let accepted = resolve(&mut state, 0.7);
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0], id);
    }

    #[test]
    fn test_reject() {
        let mut state = PreState { proposals: vec![], resolution_window_ms: 5000 };
        let id = submit(&mut state, ProposalKind::ThesisUpdate, "test", serde_json::json!({}), 60000);
        reject(&mut state, id).unwrap();
        assert_eq!(pending_count(&state), 0);
    }
}
