//! Expression Engine — slot-based prompt assembly.
//!
//! Source: 08-expression-engine.md, G1-detail-11-prompt-assembly.md
//!
//! 7 canonical slots (in order):
//!   1. SYSTEM HEADER — static, short
//!   2. OPERATING RULES — procedural memory (max 5 rules)
//!   3. ACTIVE FOCUS FRAME — ASCC/FocusState (all semantic slots)
//!   4. PARENT CONTEXT — optional, bounded (max 2 ancestors)
//!   5. ARTIFACT HANDLES — ECS refs (handles only, never inline)
//!   6. USER INPUT — raw user input
//!   7. EXECUTION DIRECTIVE — task-specific instructions
//!
//! Degradation cascade (6 steps, ordered):
//!   1. Drop parent frames beyond depth limit
//!   2. Drop non-essential ASCC slots (lowest priority first)
//!   3. Replace ASCC with checkpoint_digest
//!   4. Drop artifact handles
//!   5. Truncate user input with explicit marker
//!   6. Drop operating rules (last resort)
//!
//! Contract:
//!   - Deterministic output
//!   - Never exceeds budget
//!   - No silent truncation (all drops logged in warnings)
//!   - No reasoning, planning, or implicit summarization

use crate::expression::budget::{available_tokens, estimate_tokens};
use crate::reference::artifact::to_prompt_ref;
use crate::types::*;

// ─── Constants ──────────────────────────────────────────────────────────────

const SYSTEM_HEADER: &str = "\
You are operating within Focusa, a cognitive runtime that maintains focus over time.\n\
Follow the structured context below. Do not infer or assume context not provided.";

const DEFAULT_DIRECTIVE: &str = "\
Respond with the next best step to advance the current focus.";

const MAX_PARENT_FRAMES: usize = 2;
const MAX_RULES: usize = 5;

/// User input truncation marker — appended when input is cut.
const TRUNCATION_MARKER: &str = "\n[INPUT TRUNCATED — remaining content omitted to fit token budget]";

// ─── Output ─────────────────────────────────────────────────────────────────

/// Assembled prompt output.
#[derive(Debug, Clone)]
pub struct AssembledPrompt {
    /// The assembled prompt string (all 7 slots concatenated).
    pub content: String,
    /// Estimated token count.
    pub token_estimate: u32,
    /// Handles referenced in the prompt.
    pub handles_used: Vec<HandleId>,
    /// True if any degradation step fired.
    pub degraded: bool,
    /// Human-readable warnings about dropped/truncated content.
    pub warnings: Vec<String>,
}

// ─── Assembly Context ───────────────────────────────────────────────────────

/// All inputs needed for prompt assembly, gathered by the caller.
pub struct AssemblyInput<'a> {
    /// Active frame's FocusState.
    pub focus_state: &'a FocusState,
    /// Active frame title.
    pub frame_title: &'a str,
    /// ASCC checkpoint sections (if available; falls back to FocusState).
    pub ascc: Option<&'a AsccSections>,
    /// Parent frames (ordered root → immediate parent). Max 2 used.
    pub parent_frames: &'a [ParentContext],
    /// Procedural rules (pre-selected, max 5).
    pub rules: &'a [RuleRecord],
    /// Artifact handles referenced by this frame.
    pub handles: &'a [HandleRef],
    /// Raw user input.
    pub user_input: &'a str,
    /// Custom execution directive (None → default).
    pub directive: Option<&'a str>,
    /// Runtime config (budget limits).
    pub config: &'a FocusaConfig,
}

/// Minimal parent frame context for prompt inclusion.
pub struct ParentContext {
    pub title: String,
    pub intent: String,
    pub decisions: Vec<String>,
    pub constraints: Vec<String>,
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Assemble a prompt from cognitive state.
///
/// Simplified API that mirrors the original stub signature — builds an
/// `AssemblyInput` internally with defaults for optional fields.
pub fn assemble(
    focus_state: &FocusState,
    ascc: Option<&AsccSections>,
    rules: &[RuleRecord],
    handles: &[HandleRef],
    user_input: &str,
    config: &FocusaConfig,
) -> AssembledPrompt {
    let input = AssemblyInput {
        focus_state,
        frame_title: &focus_state.intent,
        ascc,
        parent_frames: &[],
        rules,
        handles,
        user_input,
        directive: None,
        config,
    };
    assemble_from(input)
}

/// Full assembly from explicit input context.
pub fn assemble_from(input: AssemblyInput<'_>) -> AssembledPrompt {
    let budget = available_tokens(input.config.max_prompt_tokens, input.config.reserve_for_response);
    let mut warnings: Vec<String> = Vec::new();
    let mut degraded = false;

    // ── Build each slot ──────────────────────────────────────────────

    // Slot 1: System header (always included — small, static).
    let slot_header = format!("{}\n\n", SYSTEM_HEADER);

    // Slot 2: Operating rules.
    let mut slot_rules = build_rules_slot(input.rules);

    // Slot 3: Active focus frame.
    let mut slot_focus = build_focus_slot(input.frame_title, input.focus_state, input.ascc);

    // Slot 4: Parent context.
    let mut slot_parents = build_parents_slot(input.parent_frames);

    // Slot 5: Artifact handles.
    let (mut slot_handles, mut handles_used) = build_handles_slot(input.handles);

    // Slot 6: User input (may be truncated during degradation).
    let mut slot_user = format!("USER INPUT:\n{}\n\n", input.user_input);

    // Slot 7: Execution directive.
    let directive = input.directive.unwrap_or(DEFAULT_DIRECTIVE);
    let slot_directive = format!("DIRECTIVE: {}\n", directive);

    // ── Budget check & degradation cascade ───────────────────────────

    // Fixed overhead (header + directive — never degraded).
    let fixed_tokens = estimate_tokens(&slot_header) + estimate_tokens(&slot_directive);

    // Step 1: Drop parent frames.
    if slot_total(fixed_tokens, &slot_rules, &slot_focus, &slot_parents, &slot_handles, &slot_user) > budget
        && !slot_parents.is_empty()
    {
        slot_parents = String::new();
        degraded = true;
        warnings.push("Degradation step 1: dropped parent frame context".into());
    }

    // Step 2: Drop non-essential ASCC slots (artifacts, failures, next_steps, notes).
    if slot_total(fixed_tokens, &slot_rules, &slot_focus, &slot_parents, &slot_handles, &slot_user) > budget
        && !slot_focus.is_empty()
    {
        slot_focus = build_focus_slot_reduced(input.frame_title, input.focus_state, input.ascc);
        degraded = true;
        warnings.push("Degradation step 2: dropped non-essential ASCC slots".into());
    }

    // Step 3: Replace focus with minimal digest.
    if slot_total(fixed_tokens, &slot_rules, &slot_focus, &slot_parents, &slot_handles, &slot_user) > budget
        && !slot_focus.is_empty()
    {
        slot_focus = build_focus_digest(input.frame_title, input.focus_state);
        degraded = true;
        warnings.push("Degradation step 3: replaced ASCC with checkpoint digest".into());
    }

    // Step 4: Drop artifact handles.
    if slot_total(fixed_tokens, &slot_rules, &slot_focus, &slot_parents, &slot_handles, &slot_user) > budget
        && !slot_handles.is_empty()
    {
        slot_handles = String::new();
        handles_used.clear();
        degraded = true;
        warnings.push("Degradation step 4: dropped artifact handles".into());
    }

    // Step 5: Truncate user input.
    if slot_total(fixed_tokens, &slot_rules, &slot_focus, &slot_parents, &slot_handles, &slot_user) > budget {
        let remaining = budget.saturating_sub(
            fixed_tokens
                + estimate_tokens(&slot_rules)
                + estimate_tokens(&slot_focus)
                + estimate_tokens(&slot_handles),
        );
        slot_user = truncate_user_input(input.user_input, remaining);
        degraded = true;
        warnings.push("Degradation step 5: truncated user input".into());
    }

    // Step 6: Drop rules (last resort).
    if slot_total(fixed_tokens, &slot_rules, &slot_focus, &slot_parents, &slot_handles, &slot_user) > budget
        && !slot_rules.is_empty()
    {
        slot_rules = String::new();
        degraded = true;
        warnings.push("Degradation step 6: dropped operating rules".into());
    }

    // ── Assemble final prompt ────────────────────────────────────────

    let content = format!(
        "{}{}{}{}{}{}{}",
        slot_header, slot_rules, slot_focus, slot_parents, slot_handles, slot_user, slot_directive,
    );

    let token_estimate = estimate_tokens(&content);

    AssembledPrompt {
        content,
        token_estimate,
        handles_used,
        degraded,
        warnings,
    }
}

// ─── Budget Helper ──────────────────────────────────────────────────────────

/// Sum all slot token estimates.
fn slot_total(
    fixed: u32,
    rules: &str,
    focus: &str,
    parents: &str,
    handles: &str,
    user: &str,
) -> u32 {
    fixed
        + estimate_tokens(rules)
        + estimate_tokens(focus)
        + estimate_tokens(parents)
        + estimate_tokens(handles)
        + estimate_tokens(user)
}

// ─── Slot Builders ──────────────────────────────────────────────────────────

/// Slot 2: Operating rules (max 5, ordered by weight).
fn build_rules_slot(rules: &[RuleRecord]) -> String {
    let eligible: Vec<&RuleRecord> = rules.iter().filter(|r| r.enabled).take(MAX_RULES).collect();
    if eligible.is_empty() {
        return String::new();
    }
    let mut out = String::from("RULES:\n");
    for rule in &eligible {
        out.push_str(&format!("- {}\n", rule.rule));
    }
    out.push('\n');
    out
}

/// Slot 3: Active focus frame — full version.
///
/// Uses ASCC sections if available, otherwise falls back to FocusState.
fn build_focus_slot(title: &str, state: &FocusState, ascc: Option<&AsccSections>) -> String {
    let mut out = format!("FOCUS FRAME: {}\n", title);

    match ascc {
        Some(sections) => {
            append_if_nonempty(&mut out, "INTENT", &sections.intent);
            append_if_nonempty(&mut out, "CURRENT_FOCUS", &sections.current_focus);
            append_list(&mut out, "DECISIONS", &sections.decisions);
            append_list(&mut out, "CONSTRAINTS", &sections.constraints);
            append_list(&mut out, "OPEN_QUESTIONS", &sections.open_questions);
            append_list(&mut out, "NEXT_STEPS", &sections.next_steps);
            append_list(&mut out, "RECENT_RESULTS", &sections.recent_results);
            append_list(&mut out, "FAILURES", &sections.failures);
            append_list(&mut out, "NOTES", &sections.notes);
        }
        None => {
            append_if_nonempty(&mut out, "INTENT", &state.intent);
            append_if_nonempty(&mut out, "CURRENT_STATE", &state.current_state);
            append_list(&mut out, "DECISIONS", &state.decisions);
            append_list(&mut out, "CONSTRAINTS", &state.constraints);
            append_list(&mut out, "NEXT_STEPS", &state.next_steps);
            append_list(&mut out, "FAILURES", &state.failures);
        }
    }

    out.push('\n');
    out
}

/// Slot 3 (degraded): Only high-priority sections (intent + constraints + decisions).
fn build_focus_slot_reduced(title: &str, state: &FocusState, ascc: Option<&AsccSections>) -> String {
    let mut out = format!("FOCUS FRAME: {} [REDUCED]\n", title);

    match ascc {
        Some(sections) => {
            append_if_nonempty(&mut out, "INTENT", &sections.intent);
            append_list(&mut out, "CONSTRAINTS", &sections.constraints);
            append_list(&mut out, "DECISIONS", &sections.decisions);
        }
        None => {
            append_if_nonempty(&mut out, "INTENT", &state.intent);
            append_list(&mut out, "CONSTRAINTS", &state.constraints);
            append_list(&mut out, "DECISIONS", &state.decisions);
        }
    }

    out.push('\n');
    out
}

/// Slot 3 (minimal digest): One-line intent summary only.
fn build_focus_digest(title: &str, state: &FocusState) -> String {
    if state.intent.is_empty() {
        format!("FOCUS: {}\n\n", title)
    } else {
        format!("FOCUS: {} — {}\n\n", title, state.intent)
    }
}

/// Slot 4: Parent context (max 2 ancestors, reduced sections only).
fn build_parents_slot(parents: &[ParentContext]) -> String {
    if parents.is_empty() {
        return String::new();
    }

    let mut out = String::from("PARENT CONTEXT:\n");
    for parent in parents.iter().take(MAX_PARENT_FRAMES) {
        out.push_str(&format!("FRAME: {}\n", parent.title));
        append_if_nonempty(&mut out, "INTENT", &parent.intent);
        append_list(&mut out, "DECISIONS", &parent.decisions);
        append_list(&mut out, "CONSTRAINTS", &parent.constraints);
    }
    out.push('\n');
    out
}

/// Slot 5: Artifact handles (references only, never inline content).
fn build_handles_slot(handles: &[HandleRef]) -> (String, Vec<HandleId>) {
    if handles.is_empty() {
        return (String::new(), vec![]);
    }

    let mut out = String::from("ARTIFACT REFERENCES:\n");
    let mut ids = Vec::with_capacity(handles.len());
    for handle in handles {
        out.push_str(&format!("- {}\n", to_prompt_ref(handle)));
        ids.push(handle.id);
    }
    out.push('\n');
    (out, ids)
}

/// Truncate user input to fit within a token budget, appending a marker.
fn truncate_user_input(input: &str, max_tokens: u32) -> String {
    if max_tokens == 0 {
        return format!("USER INPUT:\n{}\n\n", TRUNCATION_MARKER.trim());
    }

    // Reserve tokens for the wrapper and marker.
    let wrapper_tokens = estimate_tokens("USER INPUT:\n\n") + estimate_tokens(TRUNCATION_MARKER);
    let content_budget = max_tokens.saturating_sub(wrapper_tokens);
    let max_chars = (content_budget * 4) as usize; // inverse of estimate_tokens

    if input.len() <= max_chars {
        return format!("USER INPUT:\n{}\n\n", input);
    }

    // Find a char boundary at or before max_chars.
    let truncated = &input[..find_char_boundary(input, max_chars)];
    format!("USER INPUT:\n{}{}\n\n", truncated, TRUNCATION_MARKER)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Append a labeled single-value line if the value is non-empty.
fn append_if_nonempty(out: &mut String, label: &str, value: &str) {
    if !value.is_empty() {
        out.push_str(&format!("{}: {}\n", label, value));
    }
}

/// Append a labeled bulleted list if the list is non-empty.
fn append_list(out: &mut String, label: &str, items: &[String]) {
    if !items.is_empty() {
        out.push_str(&format!("{}:\n", label));
        for item in items {
            out.push_str(&format!("  - {}\n", item));
        }
    }
}

/// Find the largest char boundary at or before `max` in a string.
fn find_char_boundary(s: &str, max: usize) -> usize {
    if max >= s.len() {
        return s.len();
    }
    let mut i = max;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> FocusaConfig {
        FocusaConfig {
            max_prompt_tokens: 6000,
            reserve_for_response: 2000,
            ..FocusaConfig::default()
        }
    }

    fn test_state() -> FocusState {
        FocusState {
            intent: "Implement user auth".into(),
            current_state: "Working on OAuth flow".into(),
            decisions: vec!["Use JWT tokens".into()],
            constraints: vec!["Must support PKCE".into()],
            open_questions: vec![],
            next_steps: vec!["Add token refresh".into()],
            recent_results: vec![],
            failures: vec![],
            notes: vec![],
            artifacts: vec![],
        }
    }

    #[test]
    fn test_assemble_basic() {
        let config = test_config();
        let state = test_state();
        let result = assemble(&state, None, &[], &[], "Write the auth module", &config);

        assert!(result.content.contains("Focusa"));
        assert!(result.content.contains("INTENT: Implement user auth"));
        assert!(result.content.contains("USER INPUT:\nWrite the auth module"));
        assert!(result.content.contains("DIRECTIVE:"));
        assert!(!result.degraded);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_assemble_includes_rules() {
        let config = test_config();
        let state = test_state();
        let rules = vec![RuleRecord {
            id: "r1".into(),
            rule: "Prefer concise responses".into(),
            weight: 1.0,
            reinforced_count: 0,
            last_reinforced_at: chrono::Utc::now(),
            scope: RuleScope::Global,
            enabled: true,
            pinned: false,
        }];

        let result = assemble(&state, None, &rules, &[], "test", &config);
        assert!(result.content.contains("RULES:"));
        assert!(result.content.contains("Prefer concise responses"));
    }

    #[test]
    fn test_assemble_skips_disabled_rules() {
        let config = test_config();
        let state = test_state();
        let rules = vec![RuleRecord {
            id: "r1".into(),
            rule: "Should not appear".into(),
            weight: 1.0,
            reinforced_count: 0,
            last_reinforced_at: chrono::Utc::now(),
            scope: RuleScope::Global,
            enabled: false,
            pinned: false,
        }];

        let result = assemble(&state, None, &rules, &[], "test", &config);
        assert!(!result.content.contains("Should not appear"));
    }

    #[test]
    fn test_assemble_with_handles() {
        let config = test_config();
        let state = test_state();
        let handles = vec![HandleRef {
            id: uuid::Uuid::now_v7(),
            kind: HandleKind::Diff,
            label: "auth.patch".into(),
            size: 1024,
            sha256: "abc123".into(),
            created_at: chrono::Utc::now(),
            session_id: None,
            pinned: false,
        }];

        let result = assemble(&state, None, &[], &handles, "test", &config);
        assert!(result.content.contains("ARTIFACT REFERENCES:"));
        assert!(result.content.contains("[HANDLE:diff:"));
        assert!(result.content.contains("auth.patch"));
        assert_eq!(result.handles_used.len(), 1);
    }

    #[test]
    fn test_degradation_tiny_budget() {
        let config = FocusaConfig {
            max_prompt_tokens: 100,
            reserve_for_response: 50,
            ..FocusaConfig::default()
        };
        let state = test_state();
        let long_input = "x".repeat(1000);

        let result = assemble(&state, None, &[], &[], &long_input, &config);
        assert!(result.degraded);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_find_char_boundary_ascii() {
        assert_eq!(find_char_boundary("hello", 3), 3);
        assert_eq!(find_char_boundary("hello", 100), 5);
        assert_eq!(find_char_boundary("hello", 0), 0);
    }

    #[test]
    fn test_find_char_boundary_utf8() {
        let s = "héllo"; // é is 2 bytes
        assert_eq!(find_char_boundary(s, 1), 1); // 'h' boundary
        assert_eq!(find_char_boundary(s, 2), 1); // inside 'é' → backs up to 1
        assert_eq!(find_char_boundary(s, 3), 3); // after 'é'
    }

    #[test]
    fn test_empty_focus_state() {
        let config = test_config();
        let state = FocusState::default();
        let result = assemble(&state, None, &[], &[], "test", &config);
        assert!(result.content.contains("FOCUS FRAME:"));
        // Empty state should still produce valid prompt.
        assert!(result.content.contains("USER INPUT:"));
    }
}
