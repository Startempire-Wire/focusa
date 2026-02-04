//! Focus State serialization for prompt injection.
//!
//! Two serializers:
//!   - to_string_compact() — single string (FocusState)
//!   - ascc_to_messages() — structured message format (AsccSections)

use crate::types::{AsccSections, FocusState};

/// Compact string serialization of FocusState.
pub fn to_string_compact(state: &FocusState) -> String {
    let mut out = String::new();
    if !state.intent.is_empty() {
        out.push_str(&format!("INTENT: {}\n", state.intent));
    }
    if !state.current_state.is_empty() {
        out.push_str(&format!("CURRENT_STATE: {}\n", state.current_state));
    }
    if !state.decisions.is_empty() {
        out.push_str("DECISIONS:\n");
        for d in &state.decisions {
            out.push_str(&format!("  - {}\n", d));
        }
    }
    if !state.constraints.is_empty() {
        out.push_str("CONSTRAINTS:\n");
        for c in &state.constraints {
            out.push_str(&format!("  - {}\n", c));
        }
    }
    if !state.next_steps.is_empty() {
        out.push_str("NEXT_STEPS:\n");
        for n in &state.next_steps {
            out.push_str(&format!("  - {}\n", n));
        }
    }
    if !state.failures.is_empty() {
        out.push_str("FAILURES:\n");
        for f in &state.failures {
            out.push_str(&format!("  - {}\n", f));
        }
    }
    out
}

/// Serialize all ASCC sections for prompt injection.
///
/// Includes all 10 semantic slots, omitting empty sections.
pub fn ascc_to_messages(sections: &AsccSections) -> String {
    let mut out = String::new();
    if !sections.intent.is_empty() {
        out.push_str(&format!("INTENT: {}\n", sections.intent));
    }
    if !sections.current_focus.is_empty() {
        out.push_str(&format!("CURRENT_FOCUS: {}\n", sections.current_focus));
    }
    append_list(&mut out, "DECISIONS", &sections.decisions);
    append_list(&mut out, "CONSTRAINTS", &sections.constraints);
    append_list(&mut out, "OPEN_QUESTIONS", &sections.open_questions);
    append_list(&mut out, "NEXT_STEPS", &sections.next_steps);
    append_list(&mut out, "RECENT_RESULTS", &sections.recent_results);
    append_list(&mut out, "FAILURES", &sections.failures);
    append_list(&mut out, "NOTES", &sections.notes);
    // Artifact lines are serialized via the handles slot in the engine.
    out
}

/// Append a labeled bulleted list if non-empty.
fn append_list(out: &mut String, label: &str, items: &[String]) {
    if !items.is_empty() {
        out.push_str(&format!("{}:\n", label));
        for item in items {
            out.push_str(&format!("  - {}\n", item));
        }
    }
}
