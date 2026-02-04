//! Agent Constitution (ACP) — docs/16-agent-constitution.md
//! Constitution Synthesizer (CS) — docs/16-constitution-synthesizer.md
//!
//! Versioned, immutable reasoning charter.
//! SemVer. One active per agent. Never self-modifies.
//! Changes apply only to new sessions.

use crate::types::*;
use chrono::Utc;

/// Create a new constitution version.
pub fn create_version(
    state: &mut ConstitutionState,
    agent_id: &str,
    version: &str,
    principles: Vec<ConstitutionPrinciple>,
    safety_rules: Vec<String>,
    expression_rules: Vec<String>,
) -> String {
    let constitution = Constitution {
        version: version.into(),
        created_at: Utc::now(),
        agent_id: agent_id.into(),
        principles,
        self_eval_heuristics: vec![],
        autonomy_posture: "advisory".into(),
        safety_rules,
        expression_rules,
        active: false,
    };
    state.versions.push(constitution);
    version.into()
}

/// Activate a specific version. Deactivates all others.
pub fn activate_version(state: &mut ConstitutionState, version: &str) -> Result<(), String> {
    let found = state.versions.iter().any(|c| c.version == version);
    if !found {
        return Err(format!("Constitution version '{}' not found", version));
    }

    for c in &mut state.versions {
        c.active = c.version == version;
    }
    state.active_version = Some(version.into());
    Ok(())
}

/// Rollback to a previous version (one-click).
pub fn rollback(state: &mut ConstitutionState, version: &str) -> Result<(), String> {
    activate_version(state, version)
}

/// Get the active constitution.
pub fn active(state: &ConstitutionState) -> Option<&Constitution> {
    state.versions.iter().find(|c| c.active)
}

/// Get version history.
pub fn version_history(state: &ConstitutionState) -> Vec<&str> {
    state.versions.iter().map(|c| c.version.as_str()).collect()
}

// ─── Constitution Synthesizer (CS) ──────────────────────────────────────────

/// CS draft — proposed ACP revision.
#[derive(Debug, Clone)]
pub struct CsDraft {
    pub proposed_version: String,
    pub changes: Vec<CsChange>,
    pub evidence_count: usize,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CsChange {
    pub principle_id: String,
    pub change_type: CsChangeType,
    pub old_text: String,
    pub new_text: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum CsChangeType {
    Add,
    Modify,
    Remove,
    Reorder,
}

/// CS 5-step process (returns a draft for human review).
/// 1. Evidence aggregation
/// 2. Normative tension detection
/// 3. Principle impact mapping
/// 4. Candidate rewrite
/// 5. Draft assembly
///
/// Requires explicit human activation. Never auto-applies.
/// Minimum 50 tasks for analysis window.
pub fn synthesize_draft(
    state: &ConstitutionState,
    evidence: &[String],
) -> Result<CsDraft, String> {
    if evidence.len() < 50 {
        return Err(format!(
            "Insufficient evidence: {} (minimum 50 required)",
            evidence.len()
        ));
    }

    let current = active(state).ok_or("No active constitution")?;
    let next_version = bump_patch(&current.version);

    // CS produces a draft — human decides whether to apply.
    Ok(CsDraft {
        proposed_version: next_version,
        changes: vec![], // Populated by analysis engine.
        evidence_count: evidence.len(),
        created_at: Utc::now(),
    })
}

fn bump_patch(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() == 3
        && let Ok(patch) = parts[2].parse::<u32>()
    {
        return format!("{}.{}.{}", parts[0], parts[1], patch + 1);
    }
    format!("{}-next", version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_activate() {
        let mut state = ConstitutionState::default();
        create_version(&mut state, "agent-1", "1.0.0", vec![], vec![], vec![]);
        assert_eq!(state.versions.len(), 1);
        activate_version(&mut state, "1.0.0").unwrap();
        assert!(active(&state).is_some());
    }

    #[test]
    fn test_rollback() {
        let mut state = ConstitutionState::default();
        create_version(&mut state, "agent-1", "1.0.0", vec![], vec![], vec![]);
        create_version(&mut state, "agent-1", "1.1.0", vec![], vec![], vec![]);
        activate_version(&mut state, "1.1.0").unwrap();
        rollback(&mut state, "1.0.0").unwrap();
        assert_eq!(active(&state).unwrap().version, "1.0.0");
    }

    #[test]
    fn test_cs_minimum_evidence() {
        let state = ConstitutionState::default();
        let result = synthesize_draft(&state, &vec!["e".into(); 10]);
        assert!(result.is_err());
    }
}
