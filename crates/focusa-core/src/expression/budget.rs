//! Token budget management.
//!
//! Default budgets:
//!   max_prompt_tokens: 6000
//!   reserve_for_response: 2000
//!
//! Priority order for truncation (highest → lowest priority):
//!   1. Intent
//!   2. Constraints
//!   3. Decisions
//!   4. Current state
//!   5. Next steps
//!   6. Failures
//!   7. Artifacts (first to truncate)

/// Rough token estimate (4 chars ≈ 1 token).
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() as u32 + 3) / 4
}

/// Check if content fits within budget.
pub fn fits_budget(content: &str, budget: u32) -> bool {
    estimate_tokens(content) <= budget
}

/// Available tokens for prompt content.
pub fn available_tokens(max_prompt: u32, reserve: u32) -> u32 {
    max_prompt.saturating_sub(reserve)
}
