//! Procedural memory — reinforced rules/habits.
//!
//! Max 5 rules injected per turn, ordered by weight desc + scope relevance.
//! Weight decays ×0.99 per tick.
//! Scoped: global | frame | project.

use crate::types::{ExplicitMemory, FrameId, RuleRecord, RuleScope, FocusaEvent};
use chrono::Utc;

/// Reinforce a rule (increase weight).
///
/// Returns an event if the rule was found and reinforced.
pub fn reinforce(memory: &mut ExplicitMemory, rule_id: &str) -> Option<FocusaEvent> {
    if let Some(rule) = memory.procedural.iter_mut().find(|r| r.id == rule_id) {
        rule.weight += 0.1;
        rule.reinforced_count += 1;
        rule.last_reinforced_at = Utc::now();
        Some(FocusaEvent::RuleReinforced {
            rule_id: rule_id.to_string(),
            new_weight: rule.weight,
            reinforced_count: rule.reinforced_count,
        })
    } else {
        None
    }
}

/// Apply decay tick to all rules.
///
/// Returns an event with the number of rules affected.
pub fn decay_tick(memory: &mut ExplicitMemory, decay_factor: f32) -> FocusaEvent {
    let mut affected = 0usize;
    for rule in &mut memory.procedural {
        if !rule.pinned {
            rule.weight *= decay_factor;
            affected += 1;
        }
    }
    FocusaEvent::MemoryDecayTick {
        decay_factor,
        rules_affected: affected,
    }
}

/// Select top N rules for prompt injection, scoped to active frame.
pub fn select_for_prompt<'a>(
    memory: &'a ExplicitMemory,
    active_frame_id: Option<FrameId>,
    active_project_id: Option<&'a str>,
    max_rules: usize,
) -> Vec<&'a RuleRecord> {
    let mut eligible: Vec<&RuleRecord> = memory
        .procedural
        .iter()
        .filter(|r| r.enabled)
        .filter(|r| match &r.scope {
            RuleScope::Global => true,
            RuleScope::Frame(fid) => active_frame_id.as_ref() == Some(fid),
            RuleScope::Project(project_id) => active_project_id
                .map(|pid| pid == project_id)
                .unwrap_or(false),
        })
        .collect();

    eligible.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    eligible.truncate(max_rules);
    eligible
}

/// Serialize rules for prompt injection.
pub fn to_prompt_string(rules: &[&RuleRecord]) -> String {
    rules
        .iter()
        .map(|r| r.rule.clone())
        .collect::<Vec<_>>()
        .join("; ")
}
