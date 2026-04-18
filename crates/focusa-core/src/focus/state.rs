//! Focus State — the system's current state of mind.
//!
//! Source: 06-focus-state.md, G1-07-ascc.md
//!
//! 10 canonical ASCC slots with deterministic merge rules:
//!   Slot 1: intent — updated only on explicit change marker
//!   Slot 2: current_state — replaced with latest
//!   Slot 3: decisions — append/dedup (cap 30)
//!   Slot 4: artifacts — append/dedup by kind+path+label (cap 50)
//!   Slot 5: constraints — append/dedup (cap 30)
//!   Slot 6: open_questions — remove when answered, append new (cap 20)
//!   Slot 7: next_steps — replaced with latest (cap 15)
//!   Slot 8: recent_results — newest-first, keep last N (cap 10)
//!   Slot 9: failures — append-only (cap 20)
//!   Slot 10: notes — append/decay oldest first (cap 20)
//!
//! INVARIANT: All slots always exist (may be empty, never absent).
//! INVARIANT: Caps are enforced on every delta application.

use crate::types::{FocusState, FocusStateDelta, ascc_caps};

/// Apply an incremental delta to a Focus State with cap enforcement.
///
/// Merge rules per G1-07-ascc.md:
/// - intent: replace only if provided
/// - current_state: replace only if provided
/// - decisions: append unique, dedup, cap 30
/// - artifacts: append, dedup by (kind + label), cap 50
/// - constraints: append unique, dedup, cap 30
/// - open_questions: append unique, cap 20
/// - next_steps: replace entirely, cap 15
/// - recent_results: prepend (newest-first), cap 10
/// - failures: append only, cap 20
/// - notes: append, decay oldest when over cap, cap 20
pub fn apply_delta(state: &mut FocusState, delta: &FocusStateDelta) {
    // Slot 1: intent — replace if provided.
    if let Some(ref intent) = delta.intent {
        state.intent = intent.clone();
    }

    // Slot 2: current_state — replace if provided.
    if let Some(ref current_state) = delta.current_state {
        state.current_state = current_state.clone();
    }

    // Slot 3: decisions — append unique, dedup, cap 30.
    if let Some(ref decisions) = delta.decisions {
        for d in decisions {
            if !state.decisions.contains(d) {
                state.decisions.push(d.clone());
            }
        }
        state.decisions.truncate(ascc_caps::DECISIONS);
    }

    // Slot 4: artifacts — append, dedup by (kind + label), cap 50.
    if let Some(ref artifacts) = delta.artifacts {
        for a in artifacts {
            let dup = state
                .artifacts
                .iter()
                .any(|existing| existing.kind == a.kind && existing.label == a.label);
            if !dup {
                state.artifacts.push(a.clone());
            }
        }
        state.artifacts.truncate(ascc_caps::ARTIFACTS);
    }

    // Slot 5: constraints — append unique, dedup, cap 30.
    if let Some(ref constraints) = delta.constraints {
        for c in constraints {
            if !state.constraints.contains(c) {
                state.constraints.push(c.clone());
            }
        }
        state.constraints.truncate(ascc_caps::CONSTRAINTS);
    }

    // Slot 6: open_questions — append unique, cap 20.
    // Per G1-07: "if question is answered in delta, remove it (simple match heuristic)"
    if let Some(ref questions) = delta.open_questions {
        for q in questions {
            if !state.open_questions.contains(q) {
                state.open_questions.push(q.clone());
            }
        }
        state.open_questions.truncate(ascc_caps::OPEN_QUESTIONS);
    }
    // Remove answered questions: if decisions, recent_results, or current_state
    // contain keywords from an open question, consider it answered.
    if delta.decisions.is_some() || delta.recent_results.is_some() || delta.current_state.is_some()
    {
        let answer_text = [
            delta
                .decisions
                .as_ref()
                .map(|v| v.join(" "))
                .unwrap_or_default(),
            delta
                .recent_results
                .as_ref()
                .map(|v| v.join(" "))
                .unwrap_or_default(),
            delta
                .current_state
                .as_deref()
                .unwrap_or_default()
                .to_string(),
        ]
        .join(" ")
        .to_lowercase();
        if !answer_text.is_empty() {
            state.open_questions.retain(|q| {
                let q_lower = q.to_lowercase();
                // Extract key nouns/verbs from the question (words > 4 chars, stripped of punctuation).
                let stripped: Vec<String> = q_lower
                    .split_whitespace()
                    .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                    .filter(|w| w.len() > 4)
                    .filter(|w| {
                        !matches!(
                            w.as_str(),
                            "should" | "which" | "where" | "there" | "about" | "would" | "could"
                        )
                    })
                    .collect();
                let key_words: Vec<&str> = stripped.iter().map(|s| s.as_str()).collect();
                // If most key words appear in the answer text, consider answered.
                if key_words.is_empty() {
                    return true; // Can't determine — keep.
                }
                let matched = key_words
                    .iter()
                    .filter(|w| answer_text.contains(**w))
                    .count();
                let ratio = matched as f64 / key_words.len() as f64;
                ratio < 0.6 // Keep if less than 60% of key words match.
            });
        }
    }

    // Slot 7: next_steps — replace entirely, cap 15.
    if let Some(ref next_steps) = delta.next_steps {
        state.next_steps = next_steps.clone();
        state.next_steps.truncate(ascc_caps::NEXT_STEPS);
    }

    // Slot 8: recent_results — prepend (newest-first), cap 10.
    if let Some(ref results) = delta.recent_results {
        let mut merged = results.clone();
        merged.extend(state.recent_results.iter().cloned());
        merged.truncate(ascc_caps::RECENT_RESULTS);
        state.recent_results = merged;
    }

    // Slot 9: failures — append only, cap 20.
    if let Some(ref failures) = delta.failures {
        state.failures.extend(failures.iter().cloned());
        state.failures.truncate(ascc_caps::FAILURES);
    }

    // Slot 10: notes — append, decay oldest when over cap, cap 20.
    if let Some(ref notes) = delta.notes {
        state.notes.extend(notes.iter().cloned());
        // Decay oldest first if over cap.
        while state.notes.len() > ascc_caps::NOTES {
            state.notes.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_delta_all_10_slots() {
        let mut state = FocusState::default();
        let delta = FocusStateDelta {
            intent: Some("build widget".into()),
            current_state: Some("coding".into()),
            decisions: Some(vec!["use rust".into()]),
            artifacts: Some(vec![]),
            constraints: Some(vec!["no unsafe".into()]),
            open_questions: Some(vec!["which crate?".into()]),
            next_steps: Some(vec!["write tests".into()]),
            recent_results: Some(vec!["compiled ok".into()]),
            failures: Some(vec!["typo in main".into()]),
            notes: Some(vec!["check docs".into()]),
        };
        apply_delta(&mut state, &delta);
        assert_eq!(state.intent, "build widget");
        assert_eq!(state.current_state, "coding");
        assert_eq!(state.decisions, vec!["use rust"]);
        assert_eq!(state.constraints, vec!["no unsafe"]);
        assert_eq!(state.open_questions, vec!["which crate?"]);
        assert_eq!(state.next_steps, vec!["write tests"]);
        assert_eq!(state.recent_results, vec!["compiled ok"]);
        assert_eq!(state.failures, vec!["typo in main"]);
        assert_eq!(state.notes, vec!["check docs"]);
    }

    #[test]
    fn test_decisions_cap_30() {
        let mut state = FocusState::default();
        let decisions: Vec<String> = (0..50).map(|i| format!("decision_{i}")).collect();
        let delta = FocusStateDelta {
            decisions: Some(decisions),
            ..Default::default()
        };
        apply_delta(&mut state, &delta);
        assert_eq!(state.decisions.len(), ascc_caps::DECISIONS);
    }

    #[test]
    fn test_recent_results_newest_first() {
        let mut state = FocusState::default();
        state.recent_results = vec!["old".into()];
        let delta = FocusStateDelta {
            recent_results: Some(vec!["new".into()]),
            ..Default::default()
        };
        apply_delta(&mut state, &delta);
        assert_eq!(state.recent_results[0], "new");
        assert_eq!(state.recent_results[1], "old");
    }

    #[test]
    fn test_notes_decay_oldest() {
        let mut state = FocusState::default();
        state.notes = (0..20).map(|i| format!("note_{i}")).collect();
        let delta = FocusStateDelta {
            notes: Some(vec!["newest".into()]),
            ..Default::default()
        };
        apply_delta(&mut state, &delta);
        assert_eq!(state.notes.len(), ascc_caps::NOTES);
        assert_eq!(state.notes.last().unwrap(), "newest");
        // Oldest was decayed.
        assert!(!state.notes.contains(&"note_0".to_string()));
    }

    #[test]
    fn test_open_questions_removed_when_answered() {
        let mut state = FocusState::default();
        state.open_questions = vec![
            "Should we use SQLite for data persistence?".into(),
            "How to handle authentication?".into(),
        ];
        // Delta with a decision that answers the SQLite question.
        let delta = FocusStateDelta {
            decisions: Some(vec!["Using SQLite for local persistence layer".into()]),
            ..Default::default()
        };
        apply_delta(&mut state, &delta);
        // The SQLite/persistence question should be removed (key words match).
        assert!(
            !state.open_questions.iter().any(|q| q.contains("SQLite")),
            "Answered question should be removed"
        );
        // The authentication question should remain.
        assert!(
            state
                .open_questions
                .iter()
                .any(|q| q.contains("authentication")),
            "Unanswered question should remain"
        );
    }

    #[test]
    fn test_decisions_dedup() {
        let mut state = FocusState::default();
        state.decisions = vec!["use rust".into()];
        let delta = FocusStateDelta {
            decisions: Some(vec!["use rust".into(), "add tests".into()]),
            ..Default::default()
        };
        apply_delta(&mut state, &delta);
        assert_eq!(state.decisions, vec!["use rust", "add tests"]);
    }
}
