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

use crate::ascc::artifact_kind_str;
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
const TRUNCATION_MARKER: &str =
    "\n[INPUT TRUNCATED — remaining content omitted to fit token budget]";

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

impl AssembledPrompt {
    /// Create a PromptAssembled event from this assembly result.
    ///
    /// Per G1-detail-11 §Events: prompt.assembled with telemetry.
    pub fn to_event(&self, turn_id: Option<crate::types::TurnId>) -> crate::types::FocusaEvent {
        let dropped_sections: Vec<String> = self
            .warnings
            .iter()
            .filter_map(|w| {
                // Extract section name from warning messages like
                // "Degradation step 1: dropped parent frame context"
                w.strip_prefix("Degradation step ")
                    .and_then(|rest| rest.split_once(": "))
                    .map(|(_, msg)| msg.to_string())
            })
            .collect();

        crate::types::FocusaEvent::PromptAssembled {
            turn_id,
            estimated_tokens: self.token_estimate,
            budget_target: self.token_estimate, // Could be separate field in future
            dropped_sections,
            degraded: self.degraded,
        }
    }
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
    /// Active constitution principles (docs/16 §2). Injected into system header.
    pub constitution_principles: &'a [String],
    /// Safety rules from constitution (docs/16 §5).
    pub safety_rules: &'a [String],
    /// Runtime config (budget limits).
    pub config: &'a FocusaConfig,
    /// Rehydration: inline handle content up to max_tokens.
    pub rehydrate_handles: Option<u32>,
    /// Thread thesis (injected into Slot 3 per §11.5).
    pub thesis: Option<&'a crate::types::ThreadThesis>,
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
        constitution_principles: &[],
        safety_rules: &[],
        config,
        rehydrate_handles: None,
        thesis: None,
    };
    assemble_from(input)
}

/// Extract constitution principles and safety rules from state for prompt injection.
///
/// Per docs/16 §2: behavioral principles. §5: safety rules.
/// Returns (principles, safety_rules) as owned Vecs.
pub fn extract_constitution(state: &crate::types::ConstitutionState) -> (Vec<String>, Vec<String>) {
    let active = crate::constitution::active(state);
    match active {
        Some(c) => (
            c.principles.iter().map(|p| p.text.clone()).collect(),
            c.safety_rules.clone(),
        ),
        None => (vec![], vec![]),
    }
}

/// Build parent context from the focus stack for prompt assembly.
///
/// Per G1-detail-05: "Prompt assembly always includes: Active frame ASCC
/// checkpoint slots. Optionally includes parent ASCC checkpoints (bounded
/// by budget). Never includes siblings or unrelated frames in MVP."
///
/// Per G1-detail-11 §Slot 4: "Include up to N ancestors (default 2). Rules:
/// include only intent, decisions, constraints."
///
/// Returns up to `MAX_PARENT_FRAMES` (2) parent contexts, ordered root → parent.
pub fn build_parent_contexts(stack: &FocusStackState) -> Vec<ParentContext> {
    let active_id = match stack.active_id {
        Some(id) => id,
        None => return vec![],
    };

    let active_frame = match stack.frames.iter().find(|f| f.id == active_id) {
        Some(f) => f,
        None => return vec![],
    };

    // Walk up parent chain, collecting up to MAX_PARENT_FRAMES ancestors.
    let mut parents = Vec::new();
    let mut current_parent_id = active_frame.parent_id;

    while let Some(pid) = current_parent_id {
        if parents.len() >= MAX_PARENT_FRAMES {
            break;
        }
        if let Some(parent) = stack.frames.iter().find(|f| f.id == pid) {
            parents.push(ParentContext {
                title: parent.title.clone(),
                intent: parent.focus_state.intent.clone(),
                decisions: parent.focus_state.decisions.clone(),
                constraints: parent.focus_state.constraints.clone(),
            });
            current_parent_id = parent.parent_id;
        } else {
            break;
        }
    }

    // Reverse so order is root → immediate parent (spec: "root -> active").
    parents.reverse();
    parents
}

/// Full assembly from explicit input context.
pub fn assemble_from(input: AssemblyInput<'_>) -> AssembledPrompt {
    let budget = available_tokens(
        input.config.max_prompt_tokens,
        input.config.reserve_for_response,
    );
    let mut warnings: Vec<String> = Vec::new();
    let mut degraded = false;

    // ── Build each slot ──────────────────────────────────────────────

    // Slot 1: System header (always included — small, static).
    let slot_header = format!("{}\n\n", SYSTEM_HEADER);

    // Slot 1b: Constitution context. Under tight budgets, this degrades before
    // active mission semantics so intent/constraints/decisions survive longer.
    let mut slot_constitution =
        build_constitution_slot(input.constitution_principles, input.safety_rules);

    // Slot 2: Operating rules.
    let mut slot_rules = build_rules_slot(input.rules);

    // Slot 3: Active focus frame + thread thesis.
    let mut slot_focus = build_focus_slot(input.frame_title, input.focus_state, input.ascc);
    if let Some(thesis) = input.thesis
        && !thesis.primary_intent.is_empty()
    {
        slot_focus.push_str(&format!(
            "\nTHREAD THESIS:\n  Intent: {}\n",
            thesis.primary_intent
        ));
        if !thesis.secondary_goals.is_empty() {
            slot_focus.push_str(&format!("  Goals: {}\n", thesis.secondary_goals.join(", ")));
        }
        if !thesis.open_questions.is_empty() {
            slot_focus.push_str(&format!(
                "  Open questions: {}\n",
                thesis.open_questions.join("; ")
            ));
        }
        if thesis.confidence.score > 0.0 {
            slot_focus.push_str(&format!(
                "  Confidence: {:.0}%\n",
                thesis.confidence.score * 100.0
            ));
        }
    }

    // Slot 4: Parent context.
    let mut slot_parents = build_parents_slot(input.parent_frames);

    // Slot 5: Artifact handles.
    let (mut slot_handles, mut handles_used) =
        build_handles_slot(input.handles, input.rehydrate_handles);

    // Slot 6: User input (may be truncated during degradation).
    let mut slot_user = format!("USER INPUT:\n{}\n\n", input.user_input);

    // Slot 7: Execution directive.
    let directive = input.directive.unwrap_or(DEFAULT_DIRECTIVE);
    let slot_directive = format!("DIRECTIVE: {}\n", directive);

    // ── Budget check & degradation cascade ───────────────────────────

    // Fixed overhead (header + directive — never degraded).
    let fixed_tokens = estimate_tokens(&slot_header) + estimate_tokens(&slot_directive);

    // Step 0a: Reduce constitution payload.
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
        && !slot_constitution.is_empty()
    {
        slot_constitution = build_constitution_slot_limited(
            input.constitution_principles,
            input.safety_rules,
            2,
            2,
        );
        degraded = true;
        warnings.push("Degradation step 0a: reduced constitution context".into());
    }

    // Step 0b: Drop constitution entirely if still over budget.
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
        && !slot_constitution.is_empty()
    {
        slot_constitution = String::new();
        degraded = true;
        warnings.push("Degradation step 0b: dropped constitution context".into());
    }

    // Step 1: Drop parent frames.
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
        && !slot_parents.is_empty()
    {
        slot_parents = String::new();
        degraded = true;
        warnings.push("Degradation step 1: dropped parent frame context".into());
    }

    // Step 2: Drop non-essential ASCC slots (artifacts, failures, next_steps, notes).
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
        && !slot_focus.is_empty()
    {
        slot_focus = build_focus_slot_reduced(input.frame_title, input.focus_state, input.ascc);
        degraded = true;
        warnings.push("Degradation step 2: dropped non-essential ASCC slots".into());
    }

    // Step 3: Replace focus with minimal digest.
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
        && !slot_focus.is_empty()
    {
        slot_focus = build_focus_digest(input.frame_title, input.focus_state);
        degraded = true;
        warnings.push("Degradation step 3: replaced ASCC with checkpoint digest".into());
    }

    // Step 4: Drop artifact handles.
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
        && !slot_handles.is_empty()
    {
        slot_handles = String::new();
        handles_used.clear();
        degraded = true;
        warnings.push("Degradation step 4: dropped artifact handles".into());
    }

    // Step 5: Truncate user input.
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
    {
        let remaining = budget.saturating_sub(
            fixed_tokens
                + estimate_tokens(&slot_constitution)
                + estimate_tokens(&slot_rules)
                + estimate_tokens(&slot_focus)
                + estimate_tokens(&slot_handles),
        );
        slot_user = truncate_user_input(input.user_input, remaining);
        degraded = true;
        warnings.push("Degradation step 5: truncated user input".into());
    }

    // Step 6: Drop rules (last resort).
    if slot_total(
        fixed_tokens,
        &slot_constitution,
        &slot_rules,
        &slot_focus,
        &slot_parents,
        &slot_handles,
        &slot_user,
    ) > budget
        && !slot_rules.is_empty()
    {
        slot_rules = String::new();
        degraded = true;
        warnings.push("Degradation step 6: dropped operating rules".into());
    }

    // ── Assemble final prompt ────────────────────────────────────────

    let mut content = format!(
        "{}{}{}{}{}{}{}{}",
        slot_header,
        slot_constitution,
        slot_rules,
        slot_focus,
        slot_parents,
        slot_handles,
        slot_user,
        slot_directive,
    );

    // Apply redaction if enabled.
    if input.config.redaction_enabled {
        content = apply_redaction(&content, &input.config.redaction_patterns);
    }

    let mut token_estimate = estimate_tokens(&content);

    // Hard cap: prompt content must never exceed computed budget.
    // If degradation cascade still overflows (e.g. huge handle list / fixed header),
    // apply final UTF-8 safe truncation with explicit marker.
    if token_estimate > budget {
        let marker = "\n[PROMPT TRUNCATED — final hard cap applied]";
        let marker_tokens = estimate_tokens(marker);
        let content_budget = budget.saturating_sub(marker_tokens);
        let max_chars = (content_budget * 4) as usize;
        let boundary = find_char_boundary(&content, max_chars);
        content = format!("{}{}", &content[..boundary], marker);
        token_estimate = estimate_tokens(&content);
        degraded = true;
        warnings.push("Degradation step 7: final hard cap truncation applied".into());
    }

    AssembledPrompt {
        content,
        token_estimate,
        handles_used,
        degraded,
        warnings,
    }
}

/// Apply redaction patterns to content.
fn apply_redaction(content: &str, patterns: &[String]) -> String {
    let mut result = content.to_string();
    for pattern in patterns {
        // Simple regex-based replacement (using regex crate if available, otherwise skip).
        // For now, use basic string replacement for common patterns.
        if pattern.contains("\\\\d") {
            // SSN pattern: XXX-XX-XXXX
            result = result.replace(|c: char| c.is_ascii_digit(), "X");
        } else if pattern.contains("@") {
            // Email pattern: replace @ with [at]
            result = result.replace('@', "[at]");
        }
    }
    result
}

// ─── Budget Helper ──────────────────────────────────────────────────────────

/// Sum all slot token estimates.
fn slot_total(
    fixed: u32,
    constitution: &str,
    rules: &str,
    focus: &str,
    parents: &str,
    handles: &str,
    user: &str,
) -> u32 {
    fixed
        + estimate_tokens(constitution)
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
            append_list(&mut out, "OPEN_QUESTIONS", &state.open_questions);
            append_list(&mut out, "NEXT_STEPS", &state.next_steps);
            append_list(&mut out, "RECENT_RESULTS", &state.recent_results);
            append_list(&mut out, "FAILURES", &state.failures);
            append_list(&mut out, "NOTES", &state.notes);
        }
    }

    out.push('\n');
    out
}

/// Slot 3 (degraded): Only high-priority sections + pinned sections.
///
/// Per G1-07 UPDATE §Pinning: Pinned sections are immune to slot-priority eviction.
fn build_focus_slot_reduced(
    title: &str,
    state: &FocusState,
    ascc: Option<&AsccSections>,
) -> String {
    let mut out = format!("FOCUS FRAME: {} [REDUCED]\n", title);

    match ascc {
        Some(sections) => {
            // Always include high-priority sections.
            append_if_nonempty(&mut out, "INTENT", &sections.intent);
            append_list(&mut out, "CONSTRAINTS", &sections.constraints);
            append_list(&mut out, "DECISIONS", &sections.decisions);

            // Include pinned sections even in reduced mode.
            if sections.slot_meta.current_focus.pinned && !sections.current_focus.is_empty() {
                append_if_nonempty(&mut out, "CURRENT_FOCUS", &sections.current_focus);
            }
            if sections.slot_meta.open_questions.pinned && !sections.open_questions.is_empty() {
                append_list(&mut out, "OPEN_QUESTIONS", &sections.open_questions);
            }
            if sections.slot_meta.next_steps.pinned && !sections.next_steps.is_empty() {
                append_list(&mut out, "NEXT_STEPS", &sections.next_steps);
            }
            if sections.slot_meta.recent_results.pinned && !sections.recent_results.is_empty() {
                append_list(&mut out, "RECENT_RESULTS", &sections.recent_results);
            }
            if sections.slot_meta.failures.pinned && !sections.failures.is_empty() {
                append_list(&mut out, "FAILURES", &sections.failures);
            }
            if sections.slot_meta.notes.pinned && !sections.notes.is_empty() {
                append_list(&mut out, "NOTES", &sections.notes);
            }
            if sections.slot_meta.artifacts.pinned && !sections.artifacts.is_empty() {
                out.push_str("ARTIFACTS:\n");
                for a in &sections.artifacts {
                    out.push_str(&format!(
                        "  - [{}] {}\n",
                        artifact_kind_str(a.kind),
                        a.label
                    ));
                }
            }
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

fn build_constitution_slot(principles: &[String], safety_rules: &[String]) -> String {
    build_constitution_slot_limited(
        principles,
        safety_rules,
        principles.len(),
        safety_rules.len(),
    )
}

fn build_constitution_slot_limited(
    principles: &[String],
    safety_rules: &[String],
    principle_limit: usize,
    safety_limit: usize,
) -> String {
    let mut out = String::new();
    let selected_principles = principles.iter().take(principle_limit).collect::<Vec<_>>();
    let selected_safety = safety_rules.iter().take(safety_limit).collect::<Vec<_>>();

    if !selected_principles.is_empty() {
        out.push_str("CONSTITUTION PRINCIPLES:\n");
        for p in selected_principles {
            out.push_str(&format!("- {}\n", p));
        }
        out.push('\n');
    }

    if !selected_safety.is_empty() {
        out.push_str("SAFETY RULES:\n");
        for r in selected_safety {
            out.push_str(&format!("- {}\n", r));
        }
        out.push('\n');
    }

    out
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
/// Build handles slot with optional rehydration.
///
/// Per 08-expression-engine §Handle Rehydration:
/// When rehydrate_budget is Some(max_tokens), inline content up to budget.
fn build_handles_slot(
    handles: &[HandleRef],
    _rehydrate_budget: Option<u32>,
) -> (String, Vec<HandleId>) {
    if handles.is_empty() {
        return (String::new(), vec![]);
    }

    // NOTE: Full rehydration requires ReferenceStore access.
    // When _rehydrate_budget is Some, we would:
    // 1. Load handle content from store
    // 2. Truncate to max_tokens
    // 3. Inline with [REHYDRATED] marker
    // For now, always return handle refs (never inline).

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
        assert!(
            result
                .content
                .contains("USER INPUT:\nWrite the auth module")
        );
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
            tags: vec![],
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
            tags: vec![],
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

    // SMOKE TEST: Handle rehydration field exists.
    #[test]
    fn test_rehydration_field() {
        let config = test_config();
        let state = test_state();

        // Test with rehydration disabled (None).
        let input_no_rehydrate = AssemblyInput {
            focus_state: &state,
            frame_title: &state.intent,
            ascc: None,
            parent_frames: &[],
            rules: &[],
            handles: &[],
            user_input: "test",
            directive: None,
            constitution_principles: &[],
            safety_rules: &[],
            config: &config,
            rehydrate_handles: None,
            thesis: None,
        };
        let result1 = assemble_from(input_no_rehydrate);
        assert!(!result1.content.contains("[REHYDRATED]"));

        // Test with rehydration enabled (Some).
        let input_with_rehydrate = AssemblyInput {
            focus_state: &state,
            frame_title: &state.intent,
            ascc: None,
            parent_frames: &[],
            rules: &[],
            handles: &[],
            user_input: "test",
            directive: None,
            constitution_principles: &[],
            safety_rules: &[],
            config: &config,
            rehydrate_handles: Some(500), // Request 500 tokens for rehydration.
            thesis: None,
        };
        let result2 = assemble_from(input_with_rehydrate);
        // Note: Full rehydration requires ReferenceStore access.
        // This test verifies the field is accepted and passed.
        assert!(result2.content.contains("ARTIFACT REFERENCES") || result2.handles_used.is_empty());
    }
}
