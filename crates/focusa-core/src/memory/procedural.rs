//! Procedural memory — reinforced rules/habits.
//!
//! Max 5 rules injected per turn, ordered by weight desc + scope relevance.
//! Weight decays ×0.99 per tick.
//! Scoped: global | frame | project.

use crate::types::{ExplicitMemory, FrameId, RuleRecord, RuleScope};
use chrono::Utc;

/// Reinforce a rule (increase weight).
pub fn reinforce(memory: &mut ExplicitMemory, rule_id: &str) -> bool {
    if let Some(rule) = memory.procedural.iter_mut().find(|r| r.id == rule_id) {
        rule.weight += 0.1;
        rule.reinforced_count += 1;
        rule.last_reinforced_at = Utc::now();
        true
    } else {
        false
    }
}

/// Apply decay tick to all rules.
pub fn decay_tick(memory: &mut ExplicitMemory, decay_factor: f32) {
    for rule in &mut memory.procedural {
        if !rule.pinned {
            rule.weight *= decay_factor;
        }
    }
}

/// Select top N rules for prompt injection, scoped to active frame.
pub fn select_for_prompt(
    memory: &ExplicitMemory,
    active_frame_id: Option<FrameId>,
    max_rules: usize,
) -> Vec<&RuleRecord> {
    let mut eligible: Vec<&RuleRecord> = memory
        .procedural
        .iter()
        .filter(|r| r.enabled)
        .filter(|r| match &r.scope {
            RuleScope::Global => true,
            RuleScope::Frame(fid) => active_frame_id.as_ref() == Some(fid),
            RuleScope::Project(_) => true, // TODO: scope check
        })
        .collect();

    eligible.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
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
