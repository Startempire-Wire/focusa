//! Expression Engine — slot-based prompt assembly.
//!
//! 7 canonical slots (in order):
//!   1. SYSTEM HEADER — static, short
//!   2. OPERATING RULES — procedural memory (max 5 rules)
//!   3. ACTIVE FOCUS FRAME — ASCC checkpoint (all 10 slots)
//!   4. PARENT CONTEXT — optional, bounded
//!   5. ARTIFACT HANDLES — ECS refs (handles only)
//!   6. USER INPUT — raw user input
//!   7. EXECUTION DIRECTIVE — task-specific instructions
//!
//! Degradation cascade (4 steps, ordered):
//!   1. Drop lowest-priority parent frames
//!   2. Drop non-essential ASCC slots
//!   3. Truncate rehydrated handles
//!   4. Fail only as last resort

use crate::types::*;

/// Assembled prompt output.
#[derive(Debug, Clone)]
pub struct AssembledPrompt {
    pub content: String,
    pub token_estimate: u32,
    pub handles_used: Vec<HandleId>,
    pub degraded: bool,
    pub warnings: Vec<String>,
}

/// Assemble a prompt from current cognitive state.
pub fn assemble(
    _focus_state: &FocusState,
    _ascc: Option<&AsccSections>,
    _rules: &[RuleRecord],
    _handles: &[HandleRef],
    _user_input: &str,
    _config: &FocusaConfig,
) -> AssembledPrompt {
    // TODO: Implement per G1-detail-11-prompt-assembly.md
    todo!("Implement prompt assembly")
}
