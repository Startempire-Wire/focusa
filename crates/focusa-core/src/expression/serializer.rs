//! Focus State serialization for prompt injection.
//!
//! Two serializers:
//!   - to_string_compact() — single string
//!   - to_messages_slots() — structured message format

use crate::types::{AsccSections, FocusState};

/// Compact string serialization.
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

/// Serialize ASCC sections for prompt injection (messages format).
pub fn ascc_to_messages(sections: &AsccSections) -> String {
    let mut out = String::new();
    if !sections.intent.is_empty() {
        out.push_str(&format!("INTENT: {}\n", sections.intent));
    }
    if !sections.current_focus.is_empty() {
        out.push_str(&format!("CURRENT_FOCUS: {}\n", sections.current_focus));
    }
    if !sections.decisions.is_empty() {
        out.push_str("DECISIONS:\n");
        for d in &sections.decisions {
            out.push_str(&format!("  - {}\n", d));
        }
    }
    // TODO: remaining sections
    out
}
